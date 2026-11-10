//! RED/GREEN do envio para `action` guardado (pesquisa rápida, sem botão).
use super::super::tests::instalacao;
use super::super::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, OperacaoDeFormulario, ResultadoDoEnvio,
};
use super::derivar_submissao_guardada;

const HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

const RAPIDA: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "protocolo_pesquisa_rapida",
    formulario: "frmProtocoloPesquisaRapida",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["txtPesquisaRapida"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

fn action() -> String {
    format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=protocolo_pesquisa_rapida&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH}"
    )
}

fn controle() -> String {
    format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar&infra_hash={HASH}"
    )
}

#[test]
fn corpo_so_com_o_campo_do_usuario_e_debug_redigido() {
    let (url, referer) = (action(), controle());
    let s = derivar_submissao_guardada(
        &RAPIDA,
        &instalacao(),
        &url,
        &referer,
        &[("txtPesquisaRapida", "00001.000001/2026-01")],
    )
    .unwrap();
    assert_eq!(s.url(), url);
    assert_eq!(s.referer(), referer);
    assert_eq!(s.corpo(), b"txtPesquisaRapida=00001.000001%2F2026-01");
    assert_eq!(s.nomes_dos_campos(), ["txtPesquisaRapida"]);
    let debug = format!("{s:?}");
    assert!(!debug.contains(HASH), "{debug}");
}

#[test]
fn recusas_sem_rede() {
    let (url, referer) = (action(), controle());
    let derivar = |url: &str, campos: &[(&str, &str)]| {
        derivar_submissao_guardada(&RAPIDA, &instalacao(), url, &referer, campos)
    };
    let campo = |c: &str| c.to_owned();
    assert_eq!(
        derivar(&url, &[("txtPesquisaRapida", "ção")]).unwrap_err(),
        ErroSubmissaoFormulario::ValorNaoRepresentavelNaCodificacao {
            campo: campo("txtPesquisaRapida")
        }
    );
    assert_eq!(
        derivar(&url, &[("outro", "1")]).unwrap_err(),
        ErroSubmissaoFormulario::CampoForaDaListaDaOperacao {
            campo: campo("outro")
        }
    );
    assert_eq!(
        derivar(
            &url,
            &[("txtPesquisaRapida", "1"), ("txtPesquisaRapida", "2")]
        )
        .unwrap_err(),
        ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez {
            campo: campo("txtPesquisaRapida")
        }
    );
    let fora = url.replace("sei.exemplo.gov.br", "outro.exemplo.gov.br");
    assert_eq!(
        derivar(&fora, &[("txtPesquisaRapida", "1")]).unwrap_err(),
        ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao
    );
    let outra_acao = url.replace("protocolo_pesquisa_rapida", "protocolo_pesquisar");
    assert_eq!(
        derivar(&outra_acao, &[("txtPesquisaRapida", "1")]).unwrap_err(),
        ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao
    );
    let sem_hash = url.replace(&format!("&infra_hash={HASH}"), "");
    assert_eq!(
        derivar(&sem_hash, &[("txtPesquisaRapida", "1")]).unwrap_err(),
        ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico
    );
}

#[test]
fn operacao_com_botao_ou_ocultos_nao_cabe_no_envio_guardado() {
    let com_botao = OperacaoDeFormulario {
        envio: EnvioDoFormulario::BotaoDeEnvio("sbm"),
        ..RAPIDA
    };
    let (url, referer) = (action(), controle());
    assert_eq!(
        derivar_submissao_guardada(&com_botao, &instalacao(), &url, &referer, &[]).unwrap_err(),
        ErroSubmissaoFormulario::FormularioAusenteNaTela
    );
}
