//! RED/GREEN das declarações por operação `unidade` e `resultado`: janela
//! sem `lnkInfraUnidade` aberta por link de tela já conferida, e resposta
//! sem marcador observado, conferida por leitura posterior.
use super::tests::{OPERACAO, instalacao, tela_iso, url_servida};
use super::*;
use crate::erro_html::ErroHtml;
use crate::transporte::sessao::CookieDaSessao;
use ErroSubmissaoFormulario as E;
use std::io::Cursor;

const JANELA: &str = "<html><form id='frmPesquisaProtocolo' method='post' \
    action='controlador.php?acao=protocolo_pesquisar&amp;infra_hash=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'>\
    <input type='hidden' name='hdnInfraTipoPagina' value='2'>\
    <input type='hidden' name='hdnInicio' value='0'>\
    <input type='text' name='txtProtocoloPesquisa' value=''>\
    <button type='submit' name='sbmPesquisar' value='Pesquisar'>Pesquisar</button></form></html>";

const NA_ORIGEM: OperacaoDeFormulario = OperacaoDeFormulario {
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    ..OPERACAO
};

fn derivar(operacao: &OperacaoDeFormulario, html: &str) -> Result<SubmissaoDerivada, E> {
    let url = url_servida();
    derivar_submissao(
        operacao,
        &instalacao(),
        &tela_iso(&url, html),
        "UNIDADE TESTE",
        &[("txtProtocoloPesquisa", "1")],
    )
}

#[test]
fn janela_sem_unidade_exige_a_declaracao_da_operacao() {
    assert_eq!(
        derivar(&OPERACAO, JANELA).unwrap_err(),
        E::Html(ErroHtml::ausente(
            "identificacao_da_unidade_ausente_na_tela"
        ))
    );
    let s = derivar(&NA_ORIGEM, JANELA).unwrap();
    assert_eq!(
        s.nomes_dos_campos(),
        [
            "hdnInfraTipoPagina",
            "hdnInicio",
            "txtProtocoloPesquisa",
            "sbmPesquisar"
        ]
    );
}

#[test]
fn janela_que_publica_outra_unidade_continua_recusada() {
    let outra = JANELA.replacen("<html>", "<html><a id='lnkInfraUnidade'>OUTRA</a>", 1);
    assert_eq!(
        derivar(&NA_ORIGEM, &outra).unwrap_err(),
        E::UnidadeDaTelaDiferenteDaSolicitada
    );
    let mesma = JANELA.replacen(
        "<html>",
        "<html><a id='lnkInfraUnidade'>UNIDADE TESTE</a>",
        1,
    );
    assert!(derivar(&NA_ORIGEM, &mesma).is_ok());
}

fn enviar(operacao: &OperacaoDeFormulario, status: i32, corpo: &str) -> Result<RespostaDoEnvio, E> {
    let pasta = std::env::temp_dir().join(format!(
        "ies-formulario-conferencia-{}-{status}-{}",
        std::process::id(),
        corpo.len()
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    let arquivo = pasta.join("cookie");
    std::fs::write(&arquivo, "PHPSESSID=SINTETICO\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&arquivo, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let submissao = derivar(operacao, JANELA).unwrap();
    let r = enviar_submissao_via(submissao, &CookieDaSessao::ler(&arquivo), |_, _| {
        Ok(RespostaDoPost {
            resposta: RespostaWeb {
                status_code: status,
                mime: "text/html; charset=iso-8859-1".to_owned(),
                content_length: None,
                localizacao: None,
                corpo: Cursor::new(corpo.as_bytes().to_vec()),
            },
            localizacao: None,
        })
    });
    let _ = std::fs::remove_dir_all(&pasta);
    r
}

#[test]
fn resposta_sem_marcador_aceita_so_quando_a_operacao_confere_por_leitura() {
    let fecha = "<html><body><script>window.parent.fecharModal();</script></body></html>";
    assert!(matches!(
        enviar(&NA_ORIGEM, 200, fecha),
        Ok(RespostaDoEnvio::Html(_))
    ));
    let com_marcador = OperacaoDeFormulario {
        unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
        ..OPERACAO
    };
    assert_eq!(
        enviar(&com_marcador, 200, fecha).unwrap_err(),
        E::MarcadorDeResultadoAusenteNaResposta
    );
    // Status, login e MIME continuam recusas do envio.
}
