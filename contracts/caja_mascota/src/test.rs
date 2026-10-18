#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, BytesN, Env,
};

const XLM: i128 = 10_000_000; // 1 XLM = 10^7 stroops

struct Setup {
    env: Env,
    client: CajaMascotaClient<'static>,
    owner: Address,
    vet: Address,
    token: TokenClient<'static>,
}

fn setup() -> Setup {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let admin = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(admin);
    let token = TokenClient::new(&env, &sac.address());
    StellarAssetClient::new(&env, &sac.address()).mint(&owner, &(1_000 * XLM));

    let id = env.register(CajaMascota, ());
    let client = CajaMascotaClient::new(&env, &id);
    client.init(&owner, &sac.address());

    Setup {
        env,
        client,
        owner,
        vet,
        token,
    }
}

fn hash(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

#[test]
fn flujo_completo_aporte_sello_pago() {
    let s = setup();

    s.client.add_vet(&s.vet);
    assert!(s.client.is_vet(&s.vet));

    assert_eq!(s.client.deposit(&s.owner, &(200 * XLM)), 200 * XLM);
    assert_eq!(s.client.balance(), 200 * XLM);

    let h = hash(&s.env, 7);
    s.client.seal_record(&s.vet, &1, &1, &h);
    let visit = s.client.visit(&1).unwrap();
    assert_eq!(visit.pet_id, 1);
    assert_eq!(visit.vet, s.vet);
    assert_eq!(visit.hash, h);

    assert_eq!(s.client.pay_vet(&1, &(120 * XLM)), 80 * XLM);
    assert_eq!(s.token.balance(&s.vet), 120 * XLM);
    assert_eq!(s.client.paid(&1), Some(120 * XLM));
    assert_eq!(s.client.balance(), 80 * XLM);
}

#[test]
fn no_paga_una_consulta_que_no_fue_sellada() {
    let s = setup();
    s.client.add_vet(&s.vet);
    s.client.deposit(&s.owner, &(50 * XLM));

    let err = s.client.try_pay_vet(&99, &(10 * XLM)).err();
    assert_eq!(err, Some(Ok(Error::VisitNotFound)));
    assert_eq!(s.client.balance(), 50 * XLM);
}

#[test]
fn solo_una_veterinaria_verificada_puede_sellar() {
    let s = setup();
    let desconocida = Address::generate(&s.env);

    let err = s
        .client
        .try_seal_record(&desconocida, &1, &1, &hash(&s.env, 1))
        .err();
    assert_eq!(err, Some(Ok(Error::VetNotVerified)));
    assert!(s.client.visit(&1).is_none());
}

#[test]
fn una_consulta_se_paga_una_sola_vez() {
    let s = setup();
    s.client.add_vet(&s.vet);
    s.client.deposit(&s.owner, &(100 * XLM));
    s.client.seal_record(&s.vet, &1, &1, &hash(&s.env, 2));
    s.client.pay_vet(&1, &(30 * XLM));

    let err = s.client.try_pay_vet(&1, &(30 * XLM)).err();
    assert_eq!(err, Some(Ok(Error::VisitAlreadyPaid)));
    assert_eq!(s.client.balance(), 70 * XLM);
}

#[test]
fn una_consulta_no_se_sella_dos_veces() {
    let s = setup();
    s.client.add_vet(&s.vet);
    s.client.seal_record(&s.vet, &1, &1, &hash(&s.env, 3));

    let err = s
        .client
        .try_seal_record(&s.vet, &1, &1, &hash(&s.env, 4))
        .err();
    assert_eq!(err, Some(Ok(Error::VisitAlreadySealed)));
}

#[test]
fn no_paga_a_una_veterinaria_que_dejo_de_estar_verificada() {
    let s = setup();
    s.client.add_vet(&s.vet);
    s.client.deposit(&s.owner, &(100 * XLM));
    s.client.seal_record(&s.vet, &1, &1, &hash(&s.env, 5));
    s.client.remove_vet(&s.vet);

    let err = s.client.try_pay_vet(&1, &(10 * XLM)).err();
    assert_eq!(err, Some(Ok(Error::VetNotVerified)));
}

#[test]
fn rechaza_montos_invalidos_y_doble_inicializacion() {
    let s = setup();
    assert_eq!(
        s.client.try_deposit(&s.owner, &0).err(),
        Some(Ok(Error::InvalidAmount))
    );
    let otro = Address::generate(&s.env);
    assert_eq!(
        s.client.try_init(&otro, &otro).err(),
        Some(Ok(Error::AlreadyInitialized))
    );
}
