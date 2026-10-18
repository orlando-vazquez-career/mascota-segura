#![no_std]
//! CajaMascota: la cajita de ahorro de una mascota en Stellar (Soroban).
//!
//! Reglas que el contrato hace cumplir, sin confiar en ningún servidor:
//! - El dueño de la mascota es el único que puede autorizar un pago (no custodial).
//! - La cajita solo paga a veterinarias verificadas por el dueño.
//! - Solo se paga una consulta que la veterinaria selló antes (hash del registro clínico).
//! - Cada consulta se paga una sola vez.
//!
//! Los datos clínicos nunca van a la cadena: solo el sello (hash SHA-256) del registro,
//! que prueba que existe y que no fue alterado.
//!
//! En mainnet el token es USDC (Circle); en testnet se usa XLM nativo para la demo.

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, BytesN, Env,
};

/// Umbral y extensión de vida útil del estado (en ledgers de ~5 s).
const TTL_THRESHOLD: u32 = 17_280; // ~1 día
const TTL_EXTEND: u32 = 518_400; // ~30 días

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Dueño de la mascota: firma cada pago.
    Owner,
    /// Contrato del token que guarda la cajita (USDC en mainnet, XLM en testnet).
    Token,
    /// Veterinaria verificada por el dueño.
    Vet(Address),
    /// Consulta sellada, por número de consulta.
    Visit(u64),
    /// Monto pagado por una consulta.
    Paid(u64),
}

/// Una consulta sellada por la veterinaria.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Visit {
    pub pet_id: u64,
    pub vet: Address,
    /// SHA-256 del registro clínico que guarda la app fuera de la cadena.
    pub hash: BytesN<32>,
    pub timestamp: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    VetNotVerified = 4,
    VisitAlreadySealed = 5,
    VisitNotFound = 6,
    VisitAlreadyPaid = 7,
}

// Eventos: lo que el asistente de WhatsApp escucha para avisar al dueño.

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VetAdded {
    #[topic]
    pub vet: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VetRemoved {
    #[topic]
    pub vet: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deposited {
    #[topic]
    pub from: Address,
    pub amount: i128,
    pub balance: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sealed {
    #[topic]
    pub pet_id: u64,
    pub visit_id: u64,
    pub vet: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Paid {
    #[topic]
    pub visit_id: u64,
    pub vet: Address,
    pub amount: i128,
    pub balance: i128,
}

#[contract]
pub struct CajaMascota;

#[contractimpl]
impl CajaMascota {
    /// Crea la cajita. `owner` es el dueño de la mascota; `token` el activo que guarda.
    pub fn init(env: Env, owner: Address, token: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Owner) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::Token, &token);
        Self::bump_instance(&env);
        Ok(())
    }

    /// El dueño verifica una veterinaria (matrícula revisada fuera de la cadena).
    pub fn add_vet(env: Env, vet: Address) -> Result<(), Error> {
        let owner = Self::get_owner(&env)?;
        owner.require_auth();
        Self::set_vet(&env, &vet, true);
        VetAdded { vet }.publish(&env);
        Ok(())
    }

    /// El dueño quita una veterinaria de la lista.
    pub fn remove_vet(env: Env, vet: Address) -> Result<(), Error> {
        let owner = Self::get_owner(&env)?;
        owner.require_auth();
        Self::set_vet(&env, &vet, false);
        VetRemoved { vet }.publish(&env);
        Ok(())
    }

    /// Cualquiera puede aportar a la cajita (el dueño, la abuela desde España, un padrino).
    /// Devuelve el nuevo saldo.
    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<i128, Error> {
        Self::get_owner(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        from.require_auth();
        let token = Self::token_client(&env)?;
        token.transfer(&from, &env.current_contract_address(), &amount);
        Self::bump_instance(&env);
        let balance = token.balance(&env.current_contract_address());
        Deposited {
            from,
            amount,
            balance,
        }
        .publish(&env);
        Ok(balance)
    }

    /// La veterinaria sella una consulta: queda el hash del registro clínico, no el registro.
    pub fn seal_record(
        env: Env,
        vet: Address,
        pet_id: u64,
        visit_id: u64,
        hash: BytesN<32>,
    ) -> Result<(), Error> {
        Self::get_owner(&env)?;
        vet.require_auth();
        if !Self::is_vet(env.clone(), vet.clone()) {
            return Err(Error::VetNotVerified);
        }
        let key = DataKey::Visit(visit_id);
        if env.storage().persistent().has(&key) {
            return Err(Error::VisitAlreadySealed);
        }
        let visit = Visit {
            pet_id,
            vet: vet.clone(),
            hash,
            timestamp: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&key, &visit);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND);
        Self::bump_instance(&env);
        Sealed {
            pet_id,
            visit_id,
            vet,
        }
        .publish(&env);
        Ok(())
    }

    /// El dueño paga una consulta sellada. El destino sale del sello: la veterinaria que lo firmó.
    /// Devuelve el saldo que queda en la cajita.
    pub fn pay_vet(env: Env, visit_id: u64, amount: i128) -> Result<i128, Error> {
        let owner = Self::get_owner(&env)?;
        owner.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let visit: Visit = env
            .storage()
            .persistent()
            .get(&DataKey::Visit(visit_id))
            .ok_or(Error::VisitNotFound)?;
        let paid_key = DataKey::Paid(visit_id);
        if env.storage().persistent().has(&paid_key) {
            return Err(Error::VisitAlreadyPaid);
        }
        if !Self::is_vet(env.clone(), visit.vet.clone()) {
            return Err(Error::VetNotVerified);
        }
        let token = Self::token_client(&env)?;
        token.transfer(&env.current_contract_address(), &visit.vet, &amount);
        env.storage().persistent().set(&paid_key, &amount);
        env.storage()
            .persistent()
            .extend_ttl(&paid_key, TTL_THRESHOLD, TTL_EXTEND);
        Self::bump_instance(&env);
        let balance = token.balance(&env.current_contract_address());
        Paid {
            visit_id,
            vet: visit.vet,
            amount,
            balance,
        }
        .publish(&env);
        Ok(balance)
    }

    /// Saldo de la cajita (lo que el asistente convierte a "te alcanza para...").
    pub fn balance(env: Env) -> Result<i128, Error> {
        let token = Self::token_client(&env)?;
        Ok(token.balance(&env.current_contract_address()))
    }

    pub fn owner(env: Env) -> Result<Address, Error> {
        Self::get_owner(&env)
    }

    pub fn is_vet(env: Env, vet: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Vet(vet))
            .unwrap_or(false)
    }

    pub fn visit(env: Env, visit_id: u64) -> Option<Visit> {
        env.storage().persistent().get(&DataKey::Visit(visit_id))
    }

    pub fn paid(env: Env, visit_id: u64) -> Option<i128> {
        env.storage().persistent().get(&DataKey::Paid(visit_id))
    }
}

impl CajaMascota {
    fn get_owner(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(Error::NotInitialized)
    }

    fn token_client(env: &Env) -> Result<token::Client<'_>, Error> {
        let addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(Error::NotInitialized)?;
        Ok(token::Client::new(env, &addr))
    }

    fn set_vet(env: &Env, vet: &Address, verified: bool) {
        let key = DataKey::Vet(vet.clone());
        env.storage().persistent().set(&key, &verified);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND);
    }

    fn bump_instance(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
    }
}

mod test;
