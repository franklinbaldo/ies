//! RED/GREEN do fluxo de `ies unidade trocar` (#205): sequências exatas de
//! pedidos, prévia sem POST, já na unidade, a troca verificada pelo
//! Controle novo (`https://`, `http://` do GO, salto já 200) e os desvios
//! depois do POST, que saem com efeito incerto sem novo pedido.
use super::tests_apoio::*;
use super::{
    ErroTroca, EtapaDaTroca as Etapa, FalhaDepoisDoEnvio, PedidoDeTroca, SessaoDaTroca, Troca,
    trocar_unidade,
};
use crate::instalacao::Instalacao;
use crate::tela_html::HtmlServido;
use crate::transporte::FalhaTransporte;
use crate::transporte::sessao::CookieDaSessao;

fn executar(
    sigla: &str,
    confirmar: bool,
    esperada: Option<&str>,
    respostas: Vec<Resposta>,
) -> (Result<Troca, ErroTroca>, Vec<Pedido>) {
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    let cookie = arquivo_de_cookie("fluxo");
    let url = url_controle(ID_A);
    let sessao = SessaoDaTroca {
        url_do_controle: &url,
        instalacao: &instalacao,
        cookie: &CookieDaSessao::ler(&cookie),
    };
    let pedido = PedidoDeTroca {
        sigla,
        confirmar,
        unidade_esperada: esperada,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let resultado = trocar_unidade(&sessao, &pedido, &mut roteiro);
    std::fs::remove_dir_all(cookie.parent().unwrap()).unwrap();
    (resultado, roteiro.pedidos)
}

fn inicio() -> Vec<Resposta> {
    vec![html(controle("PR-A", ID_A)), html(tela())]
}

fn com(mut base: Vec<Resposta>, mais: Vec<Resposta>) -> Vec<Resposta> {
    base.extend(mais);
    base
}

fn metodos_e_urls(pedidos: &[Pedido]) -> Vec<(&'static str, String)> {
    pedidos.iter().map(|p| (p.metodo, p.url.clone())).collect()
}

fn post_da_troca() -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=infra_trocar_unidade&acao_origem=infra_trocar_unidade&infra_sistema=100000100&infra_unidade_atual={ID_A}&infra_hash={}",
        hash('c', 64)
    )
}

#[test]
fn previa_le_controle_e_tela_sem_post() {
    let (r, pedidos) = executar("PR-B", false, None, inicio());
    let troca = r.unwrap();
    assert_eq!(
        metodos_e_urls(&pedidos),
        [("GET", url_controle(ID_A)), ("GET", url_da_troca(ID_A))]
    );
    assert_eq!(troca.antes, "PR-A");
    assert_eq!(troca.pedida, "PR-B");
    assert_eq!(troca.escolhida.unwrap().id_unidade, ID_B);
    assert_eq!(troca.oferecidas.len(), 2);
    let campos = troca.campos_do_envio.unwrap();
    assert_eq!(
        campos.first().map(String::as_str),
        Some("hdnInfraTipoPagina")
    );
    assert_eq!(campos.last().map(String::as_str), Some("selInfraUnidades"));
    assert!(!troca.confirmado);
    assert!(troca.controle_novo.is_none());
}

#[test]
fn ja_na_unidade_e_sucesso_com_um_pedido() {
    for confirmar in [false, true] {
        let (r, pedidos) = executar("PR-A", confirmar, None, vec![html(controle("PR-A", ID_A))]);
        let troca = r.unwrap();
        assert_eq!(pedidos.len(), 1);
        assert!(troca.escolhida.is_none());
        assert!(troca.campos_do_envio.is_none());
        assert!(troca.controle_novo.is_none());
    }
}

#[test]
fn unidade_esperada_diferente_recusa_no_controle() {
    let (r, pedidos) = executar("PR-B", true, Some("PR-X"), inicio());
    assert_eq!(
        r.map(|_| ()),
        Err(ErroTroca::UnidadeExibidaDiferenteDaSolicitada)
    );
    assert_eq!(pedidos.len(), 1);
}

#[test]
fn sigla_nao_oferecida_recusa_depois_da_tela_sem_post() {
    let (r, pedidos) = executar("PR-X", true, None, inicio());
    match r {
        Err(ErroTroca::UnidadeNaoOferecida { oferecidas }) => assert_eq!(oferecidas.len(), 2),
        outro => panic!("{outro:?}"),
    }
    assert_eq!(pedidos.len(), 2);
}

#[test]
fn tela_com_outra_unidade_marcada_recusa_sem_post() {
    let linhas = format!(
        "{}{}",
        linha("PR-A", ID_A, false),
        linha("PR-B", ID_B, true)
    );
    let respostas = vec![
        html(controle("PR-A", ID_A)),
        html(tela_com(("PR-A", ID_A), 2, &linhas, SCRIPT)),
    ];
    let (r, pedidos) = executar("PR-B", true, None, respostas);
    assert_eq!(
        r.map(|_| ()),
        Err(ErroTroca::UnidadeMarcadaDiferenteDaAtiva)
    );
    assert_eq!(pedidos.len(), 2);
}

#[test]
fn troca_confirmada_segue_os_dois_302_e_verifica_o_controle_novo() {
    for esquema in ["https", "http"] {
        let respostas = com(
            inicio(),
            vec![
                redireciona(localizacao_com_aviso(esquema, ID_B)),
                redireciona(localizacao_limpa(esquema, ID_B)),
                html(controle("PR-B", ID_B)),
            ],
        );
        let (r, pedidos) = executar("PR-B", true, None, respostas);
        let troca = r.unwrap();
        assert_eq!(
            metodos_e_urls(&pedidos),
            [
                ("GET", url_controle(ID_A)),
                ("GET", url_da_troca(ID_A)),
                ("POST", post_da_troca()),
                ("GET", url_com_aviso(ID_B)),
                ("GET", url_limpa(ID_B)),
            ],
            "{esquema}"
        );
        let corpo = pedidos[2].corpo.as_deref().unwrap();
        assert!(
            corpo.ends_with(&format!("&selInfraUnidades={ID_B}")),
            "{corpo}"
        );
        assert!(troca.confirmado);
        let novo = troca.controle_novo.unwrap();
        assert_eq!(novo.url, url_limpa(ID_B));
        assert!(!format!("{novo:?}").contains("infra_hash"));
    }
}

#[test]
fn trocar_depende_do_controle_novo_nao_de_estado_local() {
    // O fluxo nem recebe um índice: a sigla da sessão só é conferida
    // no Controle fresco depois do POST e dos dois saltos observados.
    let respostas = com(
        inicio(),
        vec![
            redireciona(localizacao_com_aviso("https", ID_B)),
            redireciona(localizacao_limpa("https", ID_B)),
            html(controle("PR-A", ID_B)),
        ],
    );
    let (resultado, pedidos) = executar("PR-B", true, None, respostas);
    assert!(
        resultado.is_err(),
        "sem sigla nova na tela não há confirmação"
    );
    assert_eq!(
        metodos_e_urls(&pedidos),
        [
            ("GET", url_controle(ID_A)),
            ("GET", url_da_troca(ID_A)),
            ("POST", post_da_troca()),
            ("GET", url_com_aviso(ID_B)),
            ("GET", url_limpa(ID_B)),
        ]
    );
}

#[test]
fn primeiro_salto_ja_200_com_o_controle_novo_basta() {
    let respostas = com(
        inicio(),
        vec![
            redireciona(localizacao_com_aviso("https", ID_B)),
            html(controle("PR-B", ID_B)),
        ],
    );
    let (r, pedidos) = executar("PR-B", true, None, respostas);
    assert_eq!(pedidos.len(), 4);
    assert_eq!(r.unwrap().controle_novo.unwrap().url, url_com_aviso(ID_B));
}

fn incerto(respostas_depois: Vec<Resposta>) -> (ErroTroca, usize) {
    let (r, pedidos) = executar("PR-B", true, None, com(inicio(), respostas_depois));
    (r.map(|_| ()).unwrap_err(), pedidos.len())
}

#[expect(
    clippy::too_many_lines,
    reason = "Uma tabela de casos: dividir só repetiria o roteiro"
)]
#[test]
fn desvios_depois_do_post_sao_nao_confirmado_sem_novo_pedido() {
    let casos: Vec<(Vec<Resposta>, Etapa, FalhaDepoisDoEnvio, usize)> = vec![
        (
            vec![html(controle("PR-B", ID_B))],
            Etapa::Envio,
            FalhaDepoisDoEnvio::RespostaSemRedirecionamento,
            3,
        ),
        (
            vec![Err(FalhaTransporte::Rede)],
            Etapa::Envio,
            FalhaDepoisDoEnvio::Envio(crate::formulario_web::ErroSubmissaoFormulario::Transporte(
                FalhaTransporte::Rede,
            )),
            3,
        ),
        (
            vec![Ok((302, String::new(), None))],
            Etapa::Envio,
            FalhaDepoisDoEnvio::RedirecionamentoSemLocalizacao,
            3,
        ),
        (
            vec![redireciona(localizacao_com_aviso("https", ID_C))],
            Etapa::Envio,
            FalhaDepoisDoEnvio::LocalizacaoDeOutraUnidade,
            3,
        ),
        (
            vec![redireciona(
                localizacao_com_aviso("https", ID_B).replace(HOST, "outro.gov.br"),
            )],
            Etapa::Envio,
            FalhaDepoisDoEnvio::LocalizacaoForaDaFormaObservada,
            3,
        ),
        (
            vec![
                redireciona(localizacao_com_aviso("https", ID_B)),
                redireciona(localizacao_limpa("https", ID_C)),
            ],
            Etapa::PrimeiroRedirecionamento,
            FalhaDepoisDoEnvio::LocalizacaoDeOutraUnidade,
            4,
        ),
        (
            vec![
                redireciona(localizacao_com_aviso("https", ID_B)),
                Err(FalhaTransporte::Rede),
            ],
            Etapa::PrimeiroRedirecionamento,
            FalhaDepoisDoEnvio::Transporte(FalhaTransporte::Rede),
            4,
        ),
        (
            vec![
                redireciona(localizacao_com_aviso("https", ID_B)),
                redireciona(localizacao_limpa("https", ID_B)),
                redireciona(localizacao_limpa("https", ID_B)),
            ],
            Etapa::SegundoRedirecionamento,
            FalhaDepoisDoEnvio::Status(302),
            5,
        ),
        (
            vec![
                redireciona(localizacao_com_aviso("https", ID_B)),
                redireciona(localizacao_limpa("https", ID_B)),
                html(controle("PR-A", ID_B)),
            ],
            Etapa::ControleDaUnidadeNova,
            FalhaDepoisDoEnvio::UnidadeDoControleDiferenteDaPedida,
            5,
        ),
        (
            vec![
                redireciona(localizacao_com_aviso("https", ID_B)),
                redireciona(localizacao_limpa("https", ID_B)),
                html("<html>outra tela</html>".to_owned()),
            ],
            Etapa::ControleDaUnidadeNova,
            FalhaDepoisDoEnvio::ControleNaoReconhecido(
                crate::controle_reconhecido::unidade_do_controle(&HtmlServido::de_teste(
                    "<html>outra tela</html>",
                ))
                .unwrap_err(),
            ),
            5,
        ),
    ];
    for (depois, etapa, falha, n) in casos {
        let (erro, pedidos) = incerto(depois);
        assert_eq!(
            erro,
            ErroTroca::NaoConfirmado {
                etapa,
                falha: falha.clone()
            },
            "{falha:?}"
        );
        assert_eq!(pedidos, n, "{falha:?}");
    }
}
