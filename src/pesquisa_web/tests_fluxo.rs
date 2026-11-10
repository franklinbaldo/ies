//! RED/GREEN do fluxo de `ies processo buscar`: sequência exata de
//! requisições por transporte injetável, com telas sintéticas.
use super::*;
use crate::erro_html::ErroHtml;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::pesquisa_web::resposta::FaixaDaPesquisa;
use crate::pesquisa_web::tests_fixtures::{
    ORIGEM, UNIDADE, controle, href_do_menu, localizacao, menu, resultado, resultado_numa_pagina,
    sem_resultado, tela_de_pesquisa, url_controle, url_da_tela, url_do_envio, validacao,
};
use crate::processo::NumeroProcesso;
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, RespostaWeb};
use std::collections::VecDeque;
use std::io::Cursor;

/// Uma requisição vista pelo transporte: método, URL e, no POST, o corpo.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Pedido {
    pub(crate) metodo: &'static str,
    pub(crate) url: String,
    pub(crate) corpo: Option<String>,
}

/// Resposta roteirizada: status, HTML e `Location`.
pub(crate) struct Roteiro {
    respostas: VecDeque<(i32, String, Option<String>)>,
    pub(crate) pedidos: Vec<Pedido>,
    falhar_na: Option<usize>,
}

impl Roteiro {
    pub(crate) fn novo(respostas: Vec<(i32, String, Option<String>)>) -> Self {
        Self {
            respostas: respostas.into(),
            pedidos: Vec::new(),
            falhar_na: None,
        }
    }

    fn falhar_na(mut self, chamada: usize) -> Self {
        self.falhar_na = Some(chamada);
        self
    }

    fn falha_se_programada(&self) -> Result<(), FalhaTransporte> {
        if self.falhar_na == Some(self.pedidos.len()) {
            Err(FalhaTransporte::Rede)
        } else {
            Ok(())
        }
    }

    fn proxima(&mut self) -> (RespostaWeb<Cursor<Vec<u8>>>, Option<String>) {
        let (status_code, html, localizacao) = self
            .respostas
            .pop_front()
            .expect("requisição além do roteiro");
        let tamanho = html.len();
        (
            crate::transporte::testes_http::resposta(
                status_code,
                "text/html; charset=iso-8859-1",
                html.into_bytes(),
                Some(tamanho),
            ),
            localizacao,
        )
    }
}

impl EnvioDoControle for Roteiro {
    type Corpo = Cursor<Vec<u8>>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        self.pedidos.push(Pedido {
            metodo: "POST",
            url: submissao.url().to_owned(),
            corpo: Some(String::from_utf8(submissao.corpo().to_vec()).unwrap()),
        });
        self.falha_se_programada()?;
        let (resposta, localizacao) = self.proxima();
        Ok(RespostaDoPost {
            resposta,
            localizacao,
        })
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        self.pedidos.push(Pedido {
            metodo: "GET",
            url: url.to_owned(),
            corpo: None,
        });
        self.falha_se_programada()?;
        Ok(self.proxima().0)
    }
}

pub(crate) struct Pasta(std::path::PathBuf);

impl Drop for Pasta {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn cookie() -> (Pasta, std::path::PathBuf) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let pasta = std::env::temp_dir().join(format!(
        "ies-busca-fluxo-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    let arquivo = pasta.join("cookie");
    std::fs::write(&arquivo, "PHPSESSID=SINTETICO\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&arquivo, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    (Pasta(pasta), arquivo)
}

pub(crate) fn ok(html: String) -> (i32, String, Option<String>) {
    (200, html, None)
}

fn buscar(
    termo: &str,
    pagina: Option<u32>,
    unidade: Option<&str>,
    respostas: Vec<(i32, String, Option<String>)>,
) -> (Result<BuscaRealizada, ErroBuscaDeProcesso>, Vec<Pedido>) {
    let (_pasta, arquivo) = cookie();
    let url = url_controle();
    let sessao = SessaoDaBusca {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&arquivo),
    };
    let termo = NumeroProcesso::novo(termo).unwrap();
    let pedido = PedidoDeBusca {
        termo: termo.as_str(),
        pagina,
        unidade,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let r = buscar_processo(&sessao, &pedido, &mut roteiro);
    (r, roteiro.pedidos)
}

#[test]
fn tentativa_com_falha_na_segunda_requisicao_permanece_contada() {
    let (_pasta, arquivo) = cookie();
    let url = url_controle();
    let sessao = SessaoDaBusca {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&arquivo),
    };
    let termo = NumeroProcesso::novo("00001").unwrap();
    let pedido = PedidoDeBusca {
        termo: termo.as_str(),
        pagina: None,
        unidade: None,
    };
    let colheita = Colheita::nenhuma();
    let mut requisicoes = crate::transporte::contagem::ContagemDeRequisicoes::default();
    let mut roteiro = Roteiro::novo(vec![ok(controle_com_menu())]).falhar_na(2);

    let resultado = buscar_colhendo_contado::<EmProcessos, _>(
        &sessao,
        &pedido,
        &colheita,
        &mut requisicoes,
        &mut roteiro,
    );

    assert!(resultado.is_err());
    assert_eq!(roteiro.pedidos.len(), 2);
    assert_eq!(requisicoes.total(), 2);
}

pub(crate) fn controle_com_menu() -> String {
    controle(&menu(&href_do_menu()))
}

pub(crate) fn get(url: String) -> Pedido {
    Pedido {
        metodo: "GET",
        url,
        corpo: None,
    }
}

const CORPO_DA_PESQUISA: &str = "hdnInfraTipoPagina=1&sbmPesquisar=Pesquisar&rdoPesquisarEm=P\
    &selOrgaoPesquisa%5B%5D=0&hdnIdUnidade=&hdnIdAssunto=&hdnIdAssinante=&hdnIdContato=\
    &txtProtocoloPesquisa=00001\
    &hdnIdUsuarioGerador1=&hdnIdUsuarioGerador2=&hdnIdUsuarioGerador3=&hdnInicio=0";

fn post_da_pesquisa() -> Pedido {
    Pedido {
        metodo: "POST",
        url: url_do_envio(),
        corpo: Some(CORPO_DA_PESQUISA.to_owned()),
    }
}

fn tres_pedidos() -> Vec<Pedido> {
    vec![get(url_controle()), get(url_da_tela()), post_da_pesquisa()]
}

#[test]
fn redirecionamento_302_vira_processo_unico_em_tres_requisicoes() {
    let (r, pedidos) = buscar(
        "00001",
        None,
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            (302, String::new(), Some(localizacao("4321"))),
        ],
    );
    let busca = r.unwrap();
    assert_eq!(busca.unidade, UNIDADE);
    assert_eq!(
        busca.resposta,
        RespostaDaPesquisa::ProcessoUnico {
            id: crate::dominio::IdProcedimento::novo("4321").unwrap()
        }
    );
    assert_eq!(busca.requisicoes, 3);
    // A URL de trabalho é a do `Location`, resolvida; nenhum GET a ela.
    assert_eq!(
        busca.trabalho.unwrap().as_str(),
        format!("{ORIGEM}/{}", localizacao("4321"))
    );
    assert_eq!(pedidos, tres_pedidos());
}

#[test]
fn sem_resultado_e_resposta_observada_em_tres_requisicoes() {
    let (r, pedidos) = buscar(
        "00001",
        None,
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(sem_resultado("00001")),
        ],
    );
    let busca = r.unwrap();
    assert_eq!(busca.resposta, RespostaDaPesquisa::SemResultado);
    assert_eq!(busca.requisicoes, 3);
    assert_eq!(pedidos, tres_pedidos());
}

#[test]
fn lista_publica_total_com_separador_de_milhar_e_faixa() {
    let (r, pedidos) = buscar(
        "00001",
        None,
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado("00001", 1, 10, "1.234")),
        ],
    );
    let RespostaDaPesquisa::Lista(lista) = r.unwrap().resposta else {
        panic!("esperada lista");
    };
    assert_eq!(lista.total_declarado, 1234);
    assert_eq!(lista.faixa, FaixaDaPesquisa { inicio: 1, fim: 10 });
    assert_eq!(pedidos, tres_pedidos());
}

#[test]
fn pagina_2_envia_um_post_de_navegar_sem_o_botao() {
    let (r, pedidos) = buscar(
        "00001",
        Some(2),
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado("00001", 1, 10, "1.234")),
            ok(resultado("00001", 11, 20, "1.234")),
        ],
    );
    let busca = r.unwrap();
    assert_eq!(busca.requisicoes, 4);
    let RespostaDaPesquisa::Lista(lista) = busca.resposta else {
        panic!("esperada lista");
    };
    assert_eq!(
        lista.faixa,
        FaixaDaPesquisa {
            inicio: 11,
            fim: 20
        }
    );
    let mut esperados = tres_pedidos();
    esperados.push(Pedido {
        metodo: "POST",
        url: url_do_envio(),
        corpo: Some(
            CORPO_DA_PESQUISA
                .replace("sbmPesquisar=Pesquisar&", "")
                .replace("hdnInicio=0", "hdnInicio=10"),
        ),
    });
    assert_eq!(pedidos, esperados);
}

#[test]
fn pagina_fora_do_intervalo_falha_antes_do_post_de_pagina() {
    let (_r, pedidos) = buscar(
        "00001",
        Some(200),
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado("00001", 1, 10, "1.234")),
        ],
    );
    assert_eq!(pedidos, tres_pedidos());
}

#[test]
fn validacao_da_infra_e_recusa_com_mensagem_do_sei() {
    let (r, pedidos) = buscar(
        "00001",
        None,
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(validacao("Protocolo n&atilde;o informado.")),
        ],
    );
    let erro = r.unwrap_err();
    assert!(
        matches!(
            &erro,
            ErroBuscaDeProcesso::Pesquisa(ErroPesquisaProtocolo::Submissao(
                crate::formulario_web::ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(m)
            )) if m == "Protocolo não informado."
        ),
        "{erro:?}"
    );
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn pagina_desconhecida_apos_o_post_nao_e_reconhecida() {
    let (_r, pedidos) = buscar(
        "00001",
        None,
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok("<html><a id=\"lnkInfraUnidade\">UNIDADE TESTE</a><p>outra</p></html>".to_owned()),
        ],
    );
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn controle_sem_o_link_da_pesquisa_para_no_primeiro_get() {
    let (r, pedidos) = buscar("00001", None, None, vec![ok(controle(""))]);
    assert_eq!(
        r.unwrap_err(),
        ErroBuscaDeProcesso::Link(
            crate::pesquisa_web::link_da_pesquisa::ErroLinkDaPesquisa::Html(ErroHtml::ausente(
                "link_da_pesquisa_ausente_no_controle"
            ))
        )
    );
    assert_eq!(pedidos, vec![get(url_controle())]);
}

#[test]
fn link_para_outro_host_e_recusado_sem_segunda_requisicao() {
    let outro = format!("https://outro.exemplo.org/sei/{}", href_do_menu());
    let (_r, pedidos) = buscar("00001", None, None, vec![ok(controle(&menu(&outro)))]);
    assert_eq!(pedidos.len(), 1);
}

#[test]
fn unidade_solicitada_diferente_para_no_controle() {
    let (r, pedidos) = buscar("00001", None, Some("OUTRA"), vec![ok(controle_com_menu())]);
    assert_eq!(
        r.unwrap_err(),
        ErroBuscaDeProcesso::UnidadeDoControleDiferenteDaSolicitada
    );
    assert_eq!(pedidos.len(), 1);
}

#[test]
fn controle_nao_reconhecido_para_no_primeiro_get() {
    let (r, pedidos) = buscar(
        "00001",
        None,
        None,
        vec![ok(format!("<html>{}</html>", menu(&href_do_menu())))],
    );
    assert!(matches!(r, Err(ErroBuscaDeProcesso::Controle(_))));
    assert_eq!(pedidos.len(), 1);
}

#[test]
fn origem_de_outra_instalacao_nao_envia_nada() {
    let (_pasta, arquivo) = cookie();
    let url = url_controle();
    let sessao = SessaoDaBusca {
        url_do_controle: &url,
        origem: "https://outro.exemplo.org/sei",
        cookie: &CookieDaSessao::ler(&arquivo),
    };
    let termo = NumeroProcesso::novo("1").unwrap();
    let pedido = PedidoDeBusca {
        termo: termo.as_str(),
        pagina: None,
        unidade: None,
    };
    let mut roteiro = Roteiro::novo(vec![]);
    assert_eq!(
        buscar_processo(&sessao, &pedido, &mut roteiro),
        Err(ErroBuscaDeProcesso::Transporte(
            FalhaTransporte::OrigemNaoAutorizada
        ))
    );
    assert_eq!(roteiro.pedidos, vec![]);
}

#[test]
fn pagina_obtida_com_outra_faixa_e_recusada() {
    let (_r, pedidos) = buscar(
        "00001",
        Some(2),
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado("00001", 1, 10, "1.234")),
            ok(resultado("00001", 1, 10, "1.234")),
        ],
    );
    assert_eq!(pedidos.len(), 4);
}

#[test]
fn tela_de_resultado_com_outro_termo_nao_pagina() {
    let (_r, pedidos) = buscar(
        "00001",
        Some(2),
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado("00002", 1, 10, "1.234")),
        ],
    );
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn termo_acima_do_maxlength_servido_para_antes_do_post() {
    let termo = "1".repeat(51);
    let (_r, pedidos) = buscar(
        &termo,
        None,
        None,
        vec![ok(controle_com_menu()), ok(tela_de_pesquisa())],
    );
    assert_eq!(pedidos, vec![get(url_controle()), get(url_da_tela())]);
}

#[test]
fn lista_numa_pagina_publica_total_e_nao_tem_pagina_2() {
    let resposta = || ok(resultado_numa_pagina("00001", 2, "2 resultados"));
    let (r, pedidos) = buscar(
        "00001",
        None,
        None,
        vec![ok(controle_com_menu()), ok(tela_de_pesquisa()), resposta()],
    );
    let RespostaDaPesquisa::Lista(lista) = r.unwrap().resposta else {
        panic!("esperada lista");
    };
    assert_eq!(lista.total_declarado, 2);
    assert_eq!(lista.faixa, FaixaDaPesquisa { inicio: 1, fim: 2 });
    assert_eq!(pedidos, tres_pedidos());
    let (_r, pedidos) = buscar(
        "00001",
        Some(2),
        None,
        vec![ok(controle_com_menu()), ok(tela_de_pesquisa()), resposta()],
    );
    assert_eq!(pedidos, tres_pedidos());
}

#[test]
fn colheita_do_controle_nao_muda_a_sequencia_e_guarda_o_link_da_pesquisa() {
    let (_pasta, arquivo) = cookie();
    let indice =
        crate::indice_local::IndiceLocal::em(arquivo.with_file_name("indice").join("ies.sqlite3"));
    let instalacao = crate::instalacao::Instalacao::interpretar(ORIGEM).unwrap();
    let colheita = crate::colheita_do_controle::Colheita::do_cookie(
        &indice,
        &instalacao,
        "PHPSESSID=SINTETICO",
        &url_controle(),
    );
    let url = url_controle();
    let sessao = SessaoDaBusca {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&arquivo),
    };
    let termo = NumeroProcesso::novo("00001").unwrap();
    let pedido = PedidoDeBusca {
        termo: termo.as_str(),
        pagina: None,
        unidade: None,
    };
    let mut roteiro = Roteiro::novo(vec![
        ok(controle_com_menu()),
        ok(tela_de_pesquisa()),
        (302, String::new(), Some(localizacao("4321"))),
    ]);
    let r = buscar_colhendo::<EmProcessos, _>(&sessao, &pedido, &colheita, &mut roteiro);
    assert_eq!(r.unwrap().requisicoes, 3);
    assert_eq!(roteiro.pedidos, tres_pedidos());
    let geracao = crate::sessao_login::geracao_da_sessao("PHPSESSID=SINTETICO");
    let chave = crate::indice_local::ChaveDoLink {
        geracao_sessao: &geracao,
        unidade: "110000001",
        acao: "protocolo_pesquisar",
        alvo: "",
    };
    assert_eq!(
        indice.link(&instalacao, chave).map(|u| u.url().to_owned()),
        Some(url_da_tela())
    );
}
