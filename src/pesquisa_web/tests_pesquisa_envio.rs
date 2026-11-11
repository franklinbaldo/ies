//! RED/GREEN do envio da pesquisa: 200 vira tabela, 302 vira identificador,
//! sem seguir o destino. Respostas sintéticas, no máximo uma chamada.
use super::*;
use crate::dominio::IdProcedimento;
use crate::formulario_web::tests::{OPERACAO, instalacao, tela_iso, url_servida};
use crate::formulario_web::{RespostaDoPost, derivar_submissao};
use crate::pesquisa_web::ErroResultadoPesquisa;
use crate::tela_html::HtmlServido;
use crate::transporte::RespostaWeb;
use crate::transporte::sessao::CookieDaSessao;
use std::cell::Cell;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

const HASH_FORM: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const HASH_DESTINO: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

fn formulario() -> String {
    format!(
        "<html><a id='lnkInfraUnidade'>UNIDADE TESTE</a>\
         <form id='frmPesquisaProtocolo' method='post' \
         action='controlador.php?acao=protocolo_pesquisar&amp;acao_origem=protocolo_pesquisar&amp;infra_sistema=1&amp;infra_unidade_atual=5&amp;infra_hash={HASH_FORM}'>\
         <input type='hidden' name='hdnInfraTipoPagina' value='1'>\
         <input type='text' name='txtProtocoloPesquisa' maxlength='50' value=''>\
         <button type='submit' name='sbmPesquisar' value='Pesquisar'>Pesquisar</button>\
         <input type='hidden' name='hdnInicio' value='0'></form></html>"
    )
}

fn submissao() -> SubmissaoDerivada {
    let url = url_servida();
    let html = HtmlServido::de_teste(&formulario());
    derivar_submissao(
        &OPERACAO,
        &instalacao(),
        &tela_iso(&url, &html),
        Some("UNIDADE TESTE"),
        &[("txtProtocoloPesquisa", "0001")],
    )
    .unwrap()
}

struct Pasta(PathBuf);

impl Drop for Pasta {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn cookie() -> (Pasta, PathBuf) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let pasta = std::env::temp_dir().join(format!(
        "ies-pesquisa-envio-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&pasta).unwrap();
    let arquivo = pasta.join("cookie");
    fs::write(&arquivo, "PHPSESSID=SINTETICO\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
    }
    (Pasta(pasta), arquivo)
}

fn pesquisar(
    status_code: i32,
    html: &str,
    localizacao: Option<&str>,
) -> (Result<RespostaDaPesquisa, ErroPesquisaProtocolo>, usize) {
    let (_pasta, arquivo) = cookie();
    let chamadas = Cell::new(0);
    let r = pesquisar_protocolo_via(
        submissao(),
        &instalacao(),
        &CookieDaSessao::ler(&arquivo),
        |_, _| {
            chamadas.set(chamadas.get() + 1);
            Ok(RespostaDoPost {
                resposta: RespostaWeb {
                    status_code,
                    mime: "text/html; charset=iso-8859-1".to_owned(),
                    content_length: Some(html.len()),
                    localizacao: None,
                    corpo: Cursor::new(html.as_bytes().to_vec()),
                },
                localizacao: localizacao.map(str::to_owned),
            })
        },
    );
    (r, chamadas.get())
}

fn localizacao(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={id}&id_documento=&infra_sistema=1&infra_unidade_atual=5&infra_hash={HASH_DESTINO}"
    )
}

const TABELA: &str = r#"<html><div class="pesquisaBarraD">Exibindo 1 - 1 de 1</div><table class="pesquisaResultado">
  <tr class="pesquisaTituloRegistro"><td class="pesquisaTituloEsquerda">
  <a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7">00001.123456/2026-07</a>
  </td></tr></table></html>"#;

#[test]
fn resposta_200_com_tabela_vira_resultados_tipados() {
    let (r, n) = pesquisar(200, TABELA, None);
    let RespostaDaPesquisa::Lista(lista) = r.unwrap() else {
        panic!("esperada tabela");
    };
    assert_eq!(lista.processos.len(), 1);
    assert_eq!(lista.total_declarado, 1);
    assert_eq!(lista.processos[0].numero.as_str(), "00001.123456/2026-07");
    assert_eq!(n, 1);
}

#[test]
fn resposta_302_observada_vira_processo_unico_sem_segunda_chamada() {
    let (r, n) = pesquisar(302, "", Some(&localizacao("7")));
    assert_eq!(
        r.unwrap(),
        RespostaDaPesquisa::ProcessoUnico {
            id: IdProcedimento::novo("7").unwrap()
        }
    );
    assert_eq!(n, 1);
}

#[test]
fn resposta_302_fora_da_regra_e_erro_de_redirecionamento_tipado() {
    let (r, _) = pesquisar(302, "", None);
    assert_eq!(
        r.unwrap_err(),
        ErroPesquisaProtocolo::Redirecionamento(
            ErroRedirecionamentoDaPesquisa::LocalizacaoDoRedirecionamentoAusente
        )
    );
    let (_r, _) = pesquisar(302, "", Some(&localizacao("x")));
}

#[test]
fn outro_3xx_continua_falha_de_transporte_sem_interpretacao() {
    let (r, _) = pesquisar(303, "", Some(&localizacao("7")));
    assert_eq!(
        r.unwrap_err(),
        ErroPesquisaProtocolo::Submissao(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::Status(crate::transporte::StatusHttpRecusado::sem_mensagem(303))
        ))
    );
}

#[test]
fn tabela_com_marcador_mas_linha_invalida_e_erro_de_resultado() {
    let html = TABELA.replace("acao=procedimento_trabalhar", "acao=documento_imprimir_web");
    let (r, _) = pesquisar(200, &html, None);
    assert_eq!(
        r.unwrap_err(),
        ErroPesquisaProtocolo::Resposta(ErroRespostaDaPesquisa::Resultado(
            ErroResultadoPesquisa::AcaoDoLinkNaoLida(
                crate::consulta_do_controlador::ErroChaveDaConsulta::AcaoDesconhecida
            )
        ))
    );
}
