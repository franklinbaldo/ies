//! Relogin único das leituras: sequência exata de pedidos de cada caminho
//! (vencida → relogin → leitura; geração já trocada; segunda falha; sem
//! credenciais; outras falhas), com um só roteiro para login e leitura.
use super::relogin::{ContextoDoRelogin, FalhaDaLeitura, NotaDoRelogin, ler_com_relogin};
use super::tests_apoio::*;
use super::{ErroLogin, RespostaHttp, TransporteDeLogin};
use crate::configuracao::ArquivosDeAcesso;
use crate::formulario_web::SubmissaoDerivada;
use crate::indice_local::IndiceLocal;
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso as E;
use crate::transporte::envio_post::ler_tela_via;
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, RespostaWeb, StatusHttpRecusado};
use std::cell::{Cell, RefCell};
use std::io::Cursor;
use std::time::Duration;

const VENCIDO: &str = "PHPSESSID=vencida";
const AUTENTICADO: &str = "PHPSESSID=autenticada; seitreinamento=seitreinamento";

fn url_vencida() -> String {
    format!(
        "https://{HOST}/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "b".repeat(64)
    )
}

/// Login e leitura no MESMO roteiro: a ordem entre eles é verificada.
struct Compartilhado<'r>(&'r RefCell<Roteiro>);

impl TransporteDeLogin for Compartilhado<'_> {
    fn get(&mut self, url: &str, cookie: &str) -> Result<RespostaHttp, FalhaTransporte> {
        self.0.borrow_mut().get(url, cookie)
    }
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaHttp, FalhaTransporte> {
        self.0.borrow_mut().post(submissao, cookie)
    }
}

/// A leitura sintética pelo GET do Controle compartilhado (`ler_tela_via`,
/// o de `unidade *`, `processo buscar` e `documento buscar`): relê URL e
/// cookie dos arquivos a cada tentativa, com as falhas tipadas reais.
fn ler_controle(roteiro: &RefCell<Roteiro>, arquivos: &ArquivosDeAcesso) -> Result<String, E> {
    let url = Ambiente::ler(&arquivos.url_controle).trim().to_owned();
    let tela = ler_tela_via(
        &url,
        &CookieDaSessao::ler(&arquivos.cookie),
        ORIGEM,
        |url, cookie| {
            let resposta = roteiro.borrow_mut().get(url, cookie)?;
            Ok(RespostaWeb {
                status_code: i32::from(resposta.status),
                mime: resposta
                    .cabecalho("content-type")
                    .unwrap_or_default()
                    .to_owned(),
                content_length: None,
                localizacao: resposta.cabecalho("location").map(str::to_owned),
                corpo: Cursor::new(resposta.corpo),
            })
        },
    )?;
    crate::controle_reconhecido::unidade_do_controle(&tela.html).map_err(E::Controle)
}

fn login_unb() -> Vec<Result<RespostaHttp, FalhaTransporte>> {
    vec![
        Ok(r302(
            LOGIN_SIP,
            &[
                "PHPSESSID=raiz; path=/",
                "seitreinamento=seitreinamento; path=/",
            ],
        )),
        Ok(r200(&tela_acessar(), &[])),
        Ok(r302(
            "https://sei.exemplo.gov.br/sei/inicializar.php",
            &["PHPSESSID=autenticada; path=/"],
        )),
        Ok(r302(
            &url_do_controle().replace("https://sei.exemplo.gov.br/sei/", ""),
            &[],
        )),
        Ok(r200(&controle("UNIDADE-TESTE"), &[])),
    ]
}

fn sequencia_do_login() -> Vec<String> {
    vec![
        format!("GET {RAIZ}"),
        format!("GET {LOGIN_SIP}"),
        format!("POST {LOGIN_SIP}"),
        "GET https://sei.exemplo.gov.br/sei/inicializar.php".to_owned(),
        format!("GET {}", url_do_controle()),
    ]
}

fn tela_de_login() -> RespostaHttp {
    r200(&tela_acessar(), &[])
}

fn roteiro(partes: Vec<Vec<Result<RespostaHttp, FalhaTransporte>>>) -> RefCell<Roteiro> {
    RefCell::new(Roteiro::com_falhas(partes.into_iter().flatten().collect()))
}

fn vencida() -> Ambiente {
    let ambiente = Ambiente::novo();
    ambiente.gravar_sessao(&url_vencida(), VENCIDO);
    ambiente
}

type Lido = Result<(String, Option<NotaDoRelogin>), FalhaDaLeitura<E>>;

/// Executa com o contexto do ambiente e as credenciais dadas.
fn executar(
    ambiente: &Ambiente,
    credenciais: Option<&std::path::Path>,
    unidade: Option<&str>,
    roteiro: &RefCell<Roteiro>,
    mut depois_da_primeira: impl FnMut(),
) -> Lido {
    let indice = IndiceLocal::desligado();
    let contexto = ContextoDoRelogin {
        arquivos: &ambiente.arquivos,
        credenciais,
        unidade_esperada: unidade,
        espera_do_lock: Duration::from_millis(200),
        indice: &indice,
    };
    let tentativas = Cell::new(0);
    ler_com_relogin(
        &contexto,
        &mut Compartilhado(roteiro),
        &|| roteiro.borrow().pedidos.len(),
        crate::erro_publico::Causa::sessao_encerrada,
        || {
            let lido = ler_controle(roteiro, &ambiente.arquivos);
            tentativas.set(tentativas.get() + 1);
            if tentativas.get() == 1 {
                depois_da_primeira();
            }
            lido
        },
    )
}

fn sem_lock(ambiente: &Ambiente) -> bool {
    !ambiente.nomes().iter().any(|n| n == "url-controle.lock")
}

#[test]
fn so_a_falha_tipada_de_sessao_encerrada_dispara() {
    use crate::erro_publico::Causa;
    use crate::processo_obter::ErroProcessoObter;
    let status = || StatusHttpRecusado::sem_mensagem(302);
    assert!(FalhaTransporte::PaginaDeLoginRecebidaNoControle.sessao_encerrada());
    assert!(FalhaTransporte::ControleRedirecionadoParaOLogin(status()).sessao_encerrada());
    assert!(ErroProcessoObter::SessaoEncerrada.sessao_encerrada());
    // A composição pergunta ao erro interno.
    assert!(
        ErroProcessoObter::Transporte(FalhaTransporte::PaginaDeLoginRecebidaNoControle)
            .sessao_encerrada()
    );
    assert!(!ErroProcessoObter::EntradaPorIdRecusada.sessao_encerrada());
    for nao in [
        FalhaTransporte::AcessoNegado(StatusHttpRecusado::sem_mensagem(403)),
        FalhaTransporte::BloqueioDeSegurancaDaInstalacao(StatusHttpRecusado::sem_mensagem(403)),
        FalhaTransporte::Redirecionamento(status()),
        FalhaTransporte::Rede,
        FalhaTransporte::StatusHttpDeLimiteDeRequisicoes(StatusHttpRecusado::sem_mensagem(429)),
        FalhaTransporte::SessaoTrocadaDuranteALeitura,
    ] {
        assert!(!nao.sessao_encerrada(), "{nao:?}");
    }
}

#[test]
fn vencida_reloga_uma_vez_e_le_de_novo_com_a_sessao_nova() {
    let ambiente = vencida();
    let r = roteiro(vec![
        vec![Ok(tela_de_login())],
        login_unb(),
        vec![Ok(r200(&controle("UNIDADE-TESTE"), &[]))],
    ]);
    let (unidade, nota) =
        executar(&ambiente, Some(&ambiente.credenciais), None, &r, || {}).unwrap();
    assert_eq!(unidade, "UNIDADE-TESTE");
    let mut esperada = vec![format!("GET {}", url_vencida())];
    esperada.extend(sequencia_do_login());
    esperada.push(format!("GET {}", url_do_controle()));
    let r = r.into_inner();
    assert_eq!(r.sequencia(), esperada);
    assert!(r.esgotado());
    assert_eq!(r.pedidos[0].cookie, VENCIDO);
    assert_eq!(r.pedidos[6].cookie, AUTENTICADO);
    let nota = nota.unwrap();
    assert_eq!(
        nota,
        NotaDoRelogin {
            relogin: true,
            antes: 6,
            releitura: 1
        }
    );
    assert_eq!(nota.total(), 7);
    assert_eq!(Ambiente::ler(&ambiente.arquivos.cookie).trim(), AUTENTICADO);
    let marcador: serde_json::Value =
        serde_json::from_str(&Ambiente::ler(&ambiente.geracao())).unwrap();
    assert_eq!(marcador["geracao"], 1);
}

#[test]
fn geracao_trocada_por_outro_processo_usa_a_sessao_nova_sem_login() {
    let ambiente = vencida();
    let r = roteiro(vec![
        vec![Ok(tela_de_login())],
        vec![Ok(r200(&controle("UNIDADE-TESTE"), &[]))],
    ]);
    // Outro processo grava a sessão nova enquanto a primeira leitura falha.
    let outro = || ambiente.gravar_sessao(&url_do_controle(), "PHPSESSID=outro");
    let (unidade, nota) =
        executar(&ambiente, Some(&ambiente.credenciais), None, &r, outro).unwrap();
    assert_eq!(unidade, "UNIDADE-TESTE");
    let r = r.into_inner();
    assert_eq!(
        r.sequencia(),
        [
            format!("GET {}", url_vencida()),
            format!("GET {}", url_do_controle()),
        ]
    );
    assert_eq!(r.pedidos[1].cookie, "PHPSESSID=outro");
    assert_eq!(
        nota,
        Some(NotaDoRelogin {
            relogin: false,
            antes: 1,
            releitura: 1
        })
    );
    // Nenhum login: nenhum marcador de geração gravado por este processo.
    assert!(!ambiente.geracao().exists());
}

#[test]
fn segunda_falha_sai_com_o_erro_sem_novo_login() {
    let ambiente = vencida();
    let r = roteiro(vec![
        vec![Ok(tela_de_login())],
        login_unb(),
        vec![Ok(tela_de_login())],
    ]);
    let falha = executar(&ambiente, Some(&ambiente.credenciais), None, &r, || {}).unwrap_err();
    let FalhaDaLeitura::Leitura { erro, nota } = falha else {
        panic!("esperada a falha da leitura");
    };
    assert_eq!(
        erro,
        E::Transporte(FalhaTransporte::PaginaDeLoginRecebidaNoControle)
    );
    assert_eq!(
        nota,
        Some(NotaDoRelogin {
            relogin: true,
            antes: 6,
            releitura: 1
        })
    );
    let r = r.into_inner();
    let mut esperada = vec![format!("GET {}", url_vencida())];
    esperada.extend(sequencia_do_login());
    esperada.push(format!("GET {}", url_do_controle()));
    assert_eq!(r.sequencia(), esperada);
    assert!(r.esgotado());
}

#[test]
fn sem_arquivo_de_credenciais_comportamento_atual() {
    let ambiente = vencida();
    let ausente = ambiente.dir.join("credenciais-ausentes");
    for credenciais in [None, Some(ausente.as_path())] {
        let r = roteiro(vec![vec![Ok(tela_de_login())]]);
        let falha = executar(&ambiente, credenciais, None, &r, || {}).unwrap_err();
        let FalhaDaLeitura::Leitura { erro, nota } = falha else {
            panic!("esperada a falha da leitura");
        };
        assert_eq!(
            erro,
            E::Transporte(FalhaTransporte::PaginaDeLoginRecebidaNoControle)
        );
        assert_eq!(nota, None);
        assert_eq!(
            r.into_inner().sequencia(),
            [format!("GET {}", url_vencida())]
        );
    }
    assert!(sem_lock(&ambiente));
    assert_eq!(Ambiente::ler(&ambiente.arquivos.cookie).trim(), VENCIDO);
}

#[test]
fn outras_falhas_nao_relogam_nem_repetem() {
    let recusa = StatusHttpRecusado {
        status: 403,
        mensagem_do_servidor: None,
    };
    for (resposta, subcodigo) in [
        (
            Ok(status(403, None, &[], "Forbidden", "text/html")),
            "acesso_negado",
        ),
        (
            Err(FalhaTransporte::BloqueioDeSegurancaDaInstalacao(recusa)),
            "bloqueio_de_seguranca_da_instalacao",
        ),
        (Err(FalhaTransporte::Rede), "rede"),
        (
            Ok(r302("https://outro.gov.br/sip/login.php", &[])),
            "redirecionamento",
        ),
        (
            Ok(r302(
                "/sei/controlador.php?acao=procedimento_controlar",
                &[],
            )),
            "redirecionamento",
        ),
        (
            Ok(status(302, None, &[], "", "text/html")),
            "redirecionamento",
        ),
        (Ok(r302("/sip/login.php#x", &[])), "redirecionamento"),
        (
            Ok(status(301, Some(LOGIN_SIP), &[], "", "text/html")),
            "redirecionamento",
        ),
        (
            Ok(status(429, None, &[], "", "text/html")),
            "status_http_de_limite_de_requisicoes",
        ),
        (
            Ok(r200("<html><body>outra tela</body></html>", &[])),
            "tela_inesperada",
        ),
    ] {
        let ambiente = vencida();
        let r = roteiro(vec![vec![resposta]]);
        let falha = executar(&ambiente, Some(&ambiente.credenciais), None, &r, || {}).unwrap_err();
        let FalhaDaLeitura::Leitura { erro, nota } = falha else {
            panic!("esperada a falha da leitura ({subcodigo})");
        };
        assert_eq!(nota, None, "{subcodigo}");
        assert_eq!(
            r.into_inner().sequencia(),
            [format!("GET {}", url_vencida())],
            "{subcodigo}"
        );
        assert!(sem_lock(&ambiente), "{subcodigo}");
        if subcodigo != "tela_inesperada" {
            assert_eq!(erro.subcodigo(), subcodigo);
        }
    }
}

#[test]
fn relogin_recusado_sai_com_o_erro_do_login_sem_reler() {
    let ambiente = vencida();
    let r = roteiro(vec![
        vec![Ok(tela_de_login())],
        vec![
            Ok(r302(LOGIN_SIP, &["PHPSESSID=raiz; path=/"])),
            Ok(r200(&tela_acessar(), &[])),
            Ok(r200(&tela_recusada("Senha inv\u{e1}lida."), &[])),
        ],
    ]);
    let falha = executar(&ambiente, Some(&ambiente.credenciais), None, &r, || {}).unwrap_err();
    let FalhaDaLeitura::Relogin(erro) = falha else {
        panic!("esperada a falha do relogin");
    };
    assert_eq!(erro.subcodigo(), "credencial_recusada");
    let r = r.into_inner();
    assert_eq!(
        r.sequencia(),
        [
            format!("GET {}", url_vencida()),
            format!("GET {RAIZ}"),
            format!("GET {LOGIN_SIP}"),
            format!("POST {LOGIN_SIP}"),
        ]
    );
    assert_eq!(Ambiente::ler(&ambiente.arquivos.cookie).trim(), VENCIDO);
}

#[test]
fn unidade_diferente_no_relogin_nao_grava_nem_rele() {
    let ambiente = vencida();
    let r = roteiro(vec![vec![Ok(tela_de_login())], login_unb()]);
    let falha = executar(
        &ambiente,
        Some(&ambiente.credenciais),
        Some("OUTRA"),
        &r,
        || {},
    )
    .unwrap_err();
    let FalhaDaLeitura::Relogin(erro) = falha else {
        panic!("esperada a falha do relogin");
    };
    assert_eq!(erro, ErroLogin::UnidadeExibidaDiferenteDaSolicitada);
    let mut esperada = vec![format!("GET {}", url_vencida())];
    esperada.extend(sequencia_do_login());
    assert_eq!(r.into_inner().sequencia(), esperada);
    assert_eq!(Ambiente::ler(&ambiente.arquivos.cookie).trim(), VENCIDO);
    assert!(!ambiente.geracao().exists());
}

/// Os 302 do Controle para a entrada do login observada (SIP, módulo de
/// Goiás, raiz; `http://` no mesmo host): um relogin e uma releitura.
#[test]
fn controle_302_para_o_login_reloga_uma_vez_com_a_sequencia_exata() {
    let modulo = format!("https://{MODULO_GO}");
    for local in [
        LOGIN_SIP_INFRA_URL,
        modulo.as_str(),
        "/sei/",
        "http://sei.exemplo.gov.br/sip/login.php?sigla_orgao_sistema=ORGAO-A&sigla_sistema=SEI",
    ] {
        let ambiente = vencida();
        let r = roteiro(vec![
            vec![Ok(r302(local, &[]))],
            login_unb(),
            vec![Ok(r200(&controle("UNIDADE-TESTE"), &[]))],
        ]);
        let (unidade, nota) =
            executar(&ambiente, Some(&ambiente.credenciais), None, &r, || {}).unwrap();
        assert_eq!(unidade, "UNIDADE-TESTE", "{local}");
        let mut esperada = vec![format!("GET {}", url_vencida())];
        esperada.extend(sequencia_do_login());
        esperada.push(format!("GET {}", url_do_controle()));
        let r = r.into_inner();
        assert_eq!(r.sequencia(), esperada, "{local}");
        assert!(r.esgotado());
        assert_eq!(r.pedidos[0].cookie, VENCIDO);
        assert_eq!(r.pedidos[6].cookie, AUTENTICADO);
        assert_eq!(
            nota,
            Some(NotaDoRelogin {
                relogin: true,
                antes: 6,
                releitura: 1
            }),
            "{local}"
        );
    }
}

/// Segundo 302 para o login depois do relogin: sai com o erro, sem
/// segundo login.
#[test]
fn segundo_302_para_o_login_sai_com_o_erro_sem_novo_login() {
    let ambiente = vencida();
    let r = roteiro(vec![
        vec![Ok(r302(LOGIN_SIP, &[]))],
        login_unb(),
        vec![Ok(r302(LOGIN_SIP, &[]))],
    ]);
    let falha = executar(&ambiente, Some(&ambiente.credenciais), None, &r, || {}).unwrap_err();
    let FalhaDaLeitura::Leitura { erro, nota } = falha else {
        panic!("esperada a falha da leitura");
    };
    assert_eq!(erro.subcodigo(), "controle_redirecionado_para_o_login");
    assert_eq!(erro.status_http().map(|s| s.status), Some(302));
    assert_eq!(nota.map(NotaDoRelogin::total), Some(7));
    assert!(r.into_inner().esgotado());
}
