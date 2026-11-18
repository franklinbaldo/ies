//! Salvamento de despacho sem assinatura no editor clássico da UnB.
use super::{ErroSalvarNoSei, Resultado, carregar_par};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{endereco::conferir_endereco_relativo_por, tela_do_documento};
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::copia_documento::PreviaCopia;
use crate::documento::NumeroDocumento;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_obter::{
    DocumentoNaArvoreAberto, PedidoDoAnexo, PedidoDoDocumento, SessaoDoProcesso,
    abrir_documento_na_arvore, obter_foto_atual,
};
use crate::tela_html::{HtmlServido, arvore};
use crate::transporte::{FalhaTransporte, decodificar_resposta, decodificar_resposta_xml};
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
use crate::{Cli, documento_versoes::VersaoDocumento};

const PRINCIPAL: &str = "txaEditor_370";
const SECOES: [&str; 3] = ["txaEditor_368", PRINCIPAL, "txaEditor_372"];
const FIM_CABECALHO: &str = "</div>";
const INICIO_RODAPE: &str =
    r#"<hr style="border:none; padding:0; margin:5px 2px 0 2px; border-top:medium double #333" />"#;

const EDITAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::EditorSalvar,
    formulario: "frmEditor",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnVersao", "hdnIgnorarNovaVersao"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &SECOES,
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::DocumentoEditado,
    },
};

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(super) enum ErroEditorUnb {
    TelaDoDocumentoNaoPublicada,
    LinkDoEditorNaoConferido,
    ConsultaDaAssinaturaNaoConferida,
    AssinaturaNaoVerificada,
    DocumentoAssinado,
    Tela(#[from] FalhaTransporte),
    EditorForaDaForma,
    CharsetNaoSuportado,
    Formulario(#[from] ErroSubmissaoFormulario),
    FormularioDeOutroDocumento,
    ConteudoPersistidoDiferente { versao: u32 },
}

/// A foto de impressão da UnB precede o corpo com o cabeçalho de Despacho
/// centralizado e o encerra antes do separador do rodapé. Os delimitadores
/// vêm da base conferida, nunca da edição local.
fn corpo_editado<'a>(base: &str, edicao: &'a str) -> Result<&'a str, ErroSalvarNoSei> {
    let (head, body) = base
        .split_once("<body>")
        .ok_or(ErroSalvarNoSei::FotoLocalForaDaForma)?;
    if !body.trim_start().starts_with(r#"<div align="center">"#) {
        return Err(ErroSalvarNoSei::FotoLocalForaDaForma);
    }
    let (cabecalho, resto) = body
        .split_once(FIM_CABECALHO)
        .ok_or(ErroSalvarNoSei::FotoLocalForaDaForma)?;
    let (corpo, rodape) = resto
        .split_once(INICIO_RODAPE)
        .ok_or(ErroSalvarNoSei::FotoLocalForaDaForma)?;
    if rodape.contains(INICIO_RODAPE) {
        return Err(ErroSalvarNoSei::FotoLocalForaDaForma);
    }
    let prefixo = format!("{head}<body>{cabecalho}{FIM_CABECALHO}");
    let sufixo = format!("{INICIO_RODAPE}{rodape}");
    let editado = edicao
        .strip_prefix(&prefixo)
        .and_then(|html| html.strip_suffix(&sufixo))
        .ok_or(ErroSalvarNoSei::RegiaoNaoEditavelAlterada)?;
    if corpo == editado {
        return Err(ErroSalvarNoSei::BaseAlterada);
    }
    Ok(editado)
}

fn link_em_script(html: &HtmlServido, inicio: &str, fim: &str) -> Option<String> {
    let dom = arvore(html);
    let encontrados: Vec<_> = dom
        .elementos("script")
        .filter(|s| {
            !s.presente("src")
                && s.atributo("type")
                    .is_none_or(|t| t.eq_ignore_ascii_case("text/javascript"))
        })
        .flat_map(|s| {
            s.texto()
                .lines()
                .filter_map(|linha| {
                    linha
                        .trim()
                        .strip_prefix(inicio)?
                        .strip_suffix(fim)
                        .map(|l| html_escape::decode_html_entities(l).into_owned())
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let [link] = encontrados.as_slice() else {
        return None;
    };
    Some(link.clone())
}

fn assinatura(
    envio: &mut EnvioHttps,
    instalacao: &Instalacao,
    pagina: &HtmlServido,
    alvo: &DocumentoNaArvoreAberto,
) -> Result<(), ErroEditorUnb> {
    let recusa = ErroEditorUnb::ConsultaDaAssinaturaNaoConferida;
    let relativo = link_em_script(
        pagina,
        "objAjaxVerificacaoAssinatura = new infraAjaxComplementar(null,'",
        "');",
    )
    .ok_or(ErroEditorUnb::ConsultaDaAssinaturaNaoConferida)?;
    let consulta = relativo
        .strip_prefix("controlador_ajax.php?")
        .ok_or(ErroEditorUnb::ConsultaDaAssinaturaNaoConferida)?;
    let campos = ConsultaDoControlador::ler(consulta)
        .map_err(|_| ErroEditorUnb::ConsultaDaAssinaturaNaoConferida)?;
    if relativo.contains(['#', '\\', '\'', '"'])
        || !relativo.bytes().all(|b| (33..=126).contains(&b))
        || campos.valores("acao").next().is_some()
        || campos.unico("acao_ajax") != Ok("documento_verificar_assinatura")
        || campos.unico("id_documento") != Ok(alvo.id_documento.as_str())
        || !campos
            .unico("infra_hash")
            .is_ok_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(recusa);
    }
    let url = format!("{}/{relativo}", instalacao.url_base());
    let resposta = decodificar_resposta_xml(envio.get(&url, &alvo.cookie)?)?;
    let xml = crate::resposta_ajax::RespostaAjax::ler(&resposta)
        .map_err(|_| ErroEditorUnb::AssinaturaNaoVerificada)?;
    let sinais: Vec<_> = xml
        .elementos("complemento")
        .into_iter()
        .filter(|e| e.atributo("nome") == Some("SinAssinado"))
        .map(crate::resposta_ajax::Elemento::texto)
        .collect();
    match sinais.as_slice() {
        ["N"] => Ok(()),
        ["S"] => Err(ErroEditorUnb::DocumentoAssinado),
        _ => Err(ErroEditorUnb::AssinaturaNaoVerificada),
    }
}

struct Editor {
    url: String,
    html: HtmlServido,
    codificacao: crate::formulario_web::CodificacaoDaTela,
}

fn abrir_editor(
    envio: &mut EnvioHttps,
    instalacao: &Instalacao,
    alvo: &DocumentoNaArvoreAberto,
) -> Result<Editor, ErroEditorUnb> {
    let tela = tela_do_documento(
        instalacao,
        &alvo.html,
        &alvo.id_documento,
        &alvo.processo.id,
    )
    .ok_or(ErroEditorUnb::TelaDoDocumentoNaoPublicada)?;
    let pagina = decodificar_resposta(envio.get(tela.as_str(), &alvo.cookie)?)?;
    assinatura(envio, instalacao, &pagina, alvo)?;
    let relativo = link_em_script(&pagina, "janelaEditor.location.href = '", "';")
        .ok_or(ErroEditorUnb::LinkDoEditorNaoConferido)?;
    let conferir = |chave, valor| {
        conferir_endereco_relativo_por(instalacao, &relativo, AcaoDoSei::EditorMontar, chave, valor)
            .map_err(|_| ErroEditorUnb::LinkDoEditorNaoConferido)
    };
    conferir("id_procedimento", &alvo.processo.id)?;
    let url = conferir("id_documento", &alvo.id_documento)?;
    let resposta = envio.get(&url, &alvo.cookie)?;
    let codificacao =
        codificacao_da_tela(&resposta.mime).ok_or(ErroEditorUnb::CharsetNaoSuportado)?;
    let html = decodificar_resposta(resposta)?;
    Ok(Editor {
        url,
        html,
        codificacao,
    })
}

fn versao_do_editor(html: &HtmlServido) -> Option<VersaoDocumento> {
    let dom = arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, "frmEditor").ok()?;
    let oculto = |nome| {
        let controles: Vec<_> = form
            .elementos("input")
            .filter(|e| {
                e.atributo("name") == Some(nome)
                    && e.atributo("type") == Some("hidden")
                    && !e.presente("disabled")
            })
            .collect();
        let [controle] = controles.as_slice() else {
            return None;
        };
        controle.atributo("value")
    };
    if oculto("hdnIgnorarNovaVersao")? != "N" {
        return None;
    }
    VersaoDocumento::interpretar(oculto("hdnVersao")?).ok()
}

/// Lê o texto cru escapado pelo SEI, pois o DOM normaliza CR e CRLF do textarea.
fn secao(html: &HtmlServido, nome: &str) -> Option<String> {
    let aspas_duplas = format!("name=\"{nome}\"");
    let aspas_simples = format!("name='{nome}'");
    let mut secoes = html.split("<textarea").skip(1).filter_map(|trecho| {
        let (atributos, resto) = trecho.split_once('>')?;
        if !atributos.contains(&aspas_duplas) && !atributos.contains(&aspas_simples) {
            return None;
        }
        resto
            .split_once("</textarea>")
            .map(|(texto, _)| html_escape::decode_html_entities(texto).into_owned())
    });
    let unica = secoes.next()?;
    secoes.next().is_none().then_some(unica)
}

fn principal_publicado(html: &HtmlServido) -> bool {
    let dom = arvore(html);
    let mut gravaveis = Vec::new();
    for script in dom.elementos("script").filter(|s| !s.presente("src")) {
        let texto = script.texto();
        for chamada in texto.split("CKEDITOR.replace(").skip(1) {
            let Some((nome, config)) = chamada.split_once(',') else {
                return false;
            };
            let nome = nome.trim().trim_matches(['\'', '"']);
            let Some((_, readonly)) = config.split_once("\"readOnly\":") else {
                return false;
            };
            if readonly.trim_start().starts_with("false") {
                gravaveis.push(nome.to_owned());
            }
        }
    }
    gravaveis == [PRINCIPAL]
}

pub(super) fn executar(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    numero: &NumeroDocumento,
    previa: &PreviaCopia,
    instalacao: &Instalacao,
) -> Result<Resultado, ErroSalvarNoSei> {
    let (versao, base, edicao) = carregar_par(numero, previa)?;
    let editado = corpo_editado(&base, &edicao)?;
    let pedido = PedidoDoDocumento {
        numero,
        processo: None,
        unidade: cli.unidade.as_deref(),
        anexo: PedidoDoAnexo::default(),
    };
    let mut envio = EnvioHttps;
    let foto = obter_foto_atual(sessao, &pedido, indice, &mut envio)
        .map_err(ErroSalvarNoSei::LeituraDoDocumento)?;
    if foto.versao != versao {
        return Err(ErroSalvarNoSei::VersaoAlterada);
    }
    if foto.html.as_str() != base {
        return Err(ErroSalvarNoSei::BaseAlterada);
    }
    let alvo = abrir_documento_na_arvore(sessao, &pedido, indice, &mut envio)
        .map_err(ErroSalvarNoSei::LeituraDoDocumento)?;
    let editor = abrir_editor(&mut envio, instalacao, &alvo)?;
    if versao_do_editor(&editor.html) != Some(versao) || !principal_publicado(&editor.html) {
        return Err(ErroEditorUnb::EditorForaDaForma.into());
    }
    let cabecalho = secao(&editor.html, "txaEditor_368").ok_or(ErroEditorUnb::EditorForaDaForma)?;
    let rodape = secao(&editor.html, "txaEditor_372").ok_or(ErroEditorUnb::EditorForaDaForma)?;
    let submissao = derivar_submissao(
        &EDITAR,
        instalacao,
        &TelaServida {
            url: &editor.url,
            html: &editor.html,
            codificacao: editor.codificacao,
        },
        cli.unidade.as_deref(),
        &[
            ("txaEditor_368", &cabecalho),
            (PRINCIPAL, editado),
            ("txaEditor_372", &rodape),
        ],
    )
    .map_err(ErroEditorUnb::Formulario)?;
    let consulta = crate::url_observada::UrlHttps::interpretar(submissao.url())
        .and_then(|u| u.consulta)
        .and_then(|q| ConsultaDoControlador::ler(q).ok())
        .ok_or(ErroEditorUnb::FormularioDeOutroDocumento)?;
    if consulta.unico("id_documento") != Ok(alvo.id_documento.as_str())
        || consulta.unico("id_procedimento") != Ok(alvo.processo.id.as_str())
    {
        return Err(ErroEditorUnb::FormularioDeOutroDocumento.into());
    }
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| envio.post(s, c));
    let nova = previa.versao_editada;
    let aceitou =
        matches!(resposta, Ok(RespostaDoEnvio::Html(html)) if html.trim() == format!("OK {nova}"));
    let Ok(persistido) = abrir_editor(&mut envio, instalacao, &alvo) else {
        return Ok(Resultado::Incerto);
    };
    if !aceitou || versao_do_editor(&persistido.html).map(VersaoDocumento::numero) != Some(nova) {
        return Ok(Resultado::Incerto);
    }
    let salvo = secao(&persistido.html, PRINCIPAL);
    if salvo.as_deref() != Some(editado) {
        return Err(ErroEditorUnb::ConteudoPersistidoDiferente { versao: nova }.into());
    }
    Ok(Resultado::Confirmado(nova))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn recorta_a_edicao_com_os_limites_da_base_sem_alterar_o_html() {
        let antes = "<html><body><div align=\"center\"><table></table></div>";
        let depois = format!("{INICIO_RODAPE}<table></table></body></html>");
        let base = format!("{antes}<p>Antes</p>{depois}");
        let trecho = "<p title=\"sem fechamento\r\r\n<b>Exemplo";
        let editada = format!("{antes}{trecho}{depois}");
        assert_eq!(corpo_editado(&base, &editada).unwrap(), trecho);
        assert!(matches!(
            corpo_editado(&base, &editada.replace("center", "left")),
            Err(ErroSalvarNoSei::RegiaoNaoEditavelAlterada)
        ));
    }

    #[test]
    fn rele_o_textarea_preservando_caracteres_de_controle() {
        let html = HtmlServido::de_teste(
            "<textarea name='txaEditor_370'>&lt;p&gt;linha\r\r\nmais\r</textarea>",
        );
        assert_eq!(
            secao(&html, PRINCIPAL).as_deref(),
            Some("<p>linha\r\r\nmais\r")
        );
    }
}
