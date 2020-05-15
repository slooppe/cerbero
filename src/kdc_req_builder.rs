use chrono::{Duration, Utc};
use kerberos_asn1::{
    AsReq, Asn1Object, KdcReq, KerbPaPacRequest, KerberosTime, PaData,
    PrincipalName, TgsReq,
};
use kerberos_constants::{etypes, kdc_options, pa_data_types, principal_names};
use rand;
use rand::Rng;

pub struct KdcReqBuilder {
    realm: String,
    sname: Option<PrincipalName>,
    etypes: Vec<i32>,
    kdc_options: u32,
    cname: Option<PrincipalName>,
    padatas: Vec<PaData>,
    nonce: u32,
    till: KerberosTime,
    rtime: Option<KerberosTime>,
}

impl KdcReqBuilder {
    pub fn new(realm: String) -> Self {
        return Self {
            realm: realm.clone(),
            sname: Some(PrincipalName {
                name_type: principal_names::NT_PRINCIPAL,
                name_string: vec!["krbtgt".into(), realm],
            }),
            etypes: supported_etypes(),
            kdc_options: kdc_options::FORWARDABLE
                | kdc_options::RENEWABLE
                | kdc_options::CANONICALIZE
                | kdc_options::RENEWABLE_OK,
            cname: None,
            padatas: Vec::new(),
            nonce: rand::thread_rng().gen(),
            till: Utc::now()
                .checked_add_signed(Duration::weeks(20 * 52))
                .unwrap()
                .into(),
            rtime: Some(
                Utc::now()
                    .checked_add_signed(Duration::weeks(20 * 52))
                    .unwrap()
                    .into(),
            ),
        };
    }

    pub fn etypes(mut self, etypes: Vec<i32>) -> Self {
        self.etypes = etypes;
        self
    }

    pub fn cname(mut self, cname: Option<PrincipalName>) -> Self {
        self.cname = cname;
        self
    }

    pub fn sname(mut self, sname: Option<PrincipalName>) -> Self {
        self.sname = sname;
        self
    }

    pub fn username(self, username: String) -> Self {
        self.cname(Some(PrincipalName {
            name_type: principal_names::NT_PRINCIPAL,
            name_string: vec![username],
        }))
    }

    pub fn push_padata(mut self, padata: PaData) -> Self {
        self.padatas.push(padata);
        self
    }

    pub fn padatas(mut self, padatas: Vec<PaData>) -> Self {
        self.padatas = padatas;
        self
    }

    pub fn request_pac(self) -> Self {
        self.push_padata(PaData::new(
            pa_data_types::PA_PAC_REQUEST,
            KerbPaPacRequest::new(true).build(),
        ))
    }

    pub fn build(self) -> KdcReq {
        let mut req = KdcReq::default();

        req.req_body.kdc_options = self.kdc_options.into();
        req.req_body.cname = self.cname;
        req.req_body.realm = self.realm;
        req.req_body.sname = self.sname;
        req.req_body.till = self.till;
        req.req_body.rtime = self.rtime;
        req.req_body.nonce = self.nonce;
        req.req_body.etypes = self.etypes;

        if self.padatas.len() > 0 {
            req.padata = Some(self.padatas);
        }

        return req;
    }

    pub fn build_as_req(self) -> AsReq {
        self.build().into()
    }

    pub fn build_tgs_req(self) -> TgsReq {
        self.build().into()
    }
}

fn supported_etypes() -> Vec<i32> {
    vec![
        etypes::RC4_HMAC,
        etypes::AES128_CTS_HMAC_SHA1_96,
        etypes::AES256_CTS_HMAC_SHA1_96,
    ]
}
