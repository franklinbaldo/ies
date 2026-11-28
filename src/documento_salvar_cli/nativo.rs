//! Salvamento no editor publicado para uma minuta sem assinatura.
use super::{ErroSalvarNoSei, Resultado, carregar_par, html_para_envio};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{
    EdicaoNaBarra, ErroBarraDoDocumento, endereco::conferir_endereco_relativo_por,
    ler_barra_do_documento, tela_do_documento,
};
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

#[path = "classico.rs"]
mod classico;
use classico::{principal_publicado, secao, versao_do_editor};

#[path = "ckeditor.rs"]
mod ckeditor;
#[path = "copia_renderizada.rs"]
mod copia_renderizada;
use copia_renderizada::Secao;

#[path = "nativo_releitura.rs"]
mod releitura;
use releitura::{RespostaDoPost, classificar_resposta_do_post, reabrir_apos_aceite};

const EDITAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::EditorSalvar,
    formulario: "frmEditor",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnVersao", "hdnIgnorarNovaVersao"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["txaEditor_*"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::DocumentoEditado,
    },
};

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(super) enum ErroEditorNativo {
    TelaDoDocumentoNaoPublicada,
    LinkDoEditorNaoConferido,
    BarraDoDocumento(#[from] ErroBarraDoDocumento),
    ConsultaDaAssinaturaNaoConferida,
    AssinaturaNaoVerificada,
    DocumentoAssinado,
    Tela(#[from] FalhaTransporte),
    EditorForaDaForma,
    CharsetNaoSuportado,
    Formulario(#[from] ErroSubmissaoFormulario),
    FormularioDeOutroDocumento,
    EnvioSemAceite(#[source] ErroSubmissaoFormulario),
    JsonDoEditor(#[source] serde_json::Error),
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
    envio: &mut impl EnvioDoControle,
    instalacao: &Instalacao,
    pagina: &HtmlServido,
    alvo: &DocumentoNaArvoreAberto,
) -> Result<(), ErroEditorNativo> {
    let recusa = ErroEditorNativo::ConsultaDaAssinaturaNaoConferida;
    let relativo = link_em_script(
        pagina,
        "objAjaxVerificacaoAssinatura = new infraAjaxComplementar(null,'",
        "');",
    )
    .ok_or(ErroEditorNativo::ConsultaDaAssinaturaNaoConferida)?;
    let consulta = relativo
        .strip_prefix("controlador_ajax.php?")
        .ok_or(ErroEditorNativo::ConsultaDaAssinaturaNaoConferida)?;
    let campos = ConsultaDoControlador::ler(consulta)
        .map_err(|_| ErroEditorNativo::ConsultaDaAssinaturaNaoConferida)?;
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
        .map_err(|_| ErroEditorNativo::AssinaturaNaoVerificada)?;
    let sinais: Vec<_> = xml
        .elementos("complemento")
        .into_iter()
        .filter(|e| e.atributo("nome") == Some("SinAssinado"))
        .map(crate::resposta_ajax::Elemento::texto)
        .collect();
    match sinais.as_slice() {
        ["N"] => Ok(()),
        ["S"] => Err(ErroEditorNativo::DocumentoAssinado),
        _ => Err(ErroEditorNativo::AssinaturaNaoVerificada),
    }
}

struct Editor {
    url: String,
    html: HtmlServido,
    codificacao: crate::formulario_web::CodificacaoDaTela,
}

// Um endereço num script não comprova que a ação foi oferecida na barra.
fn conferir_oferta(
    instalacao: &Instalacao,
    html: &str,
    documento: &str,
    processo: &str,
) -> Result<(), ErroEditorNativo> {
    match ler_barra_do_documento(instalacao, html, documento, processo)?.edicao {
        EdicaoNaBarra::NaoAssinado => Ok(()),
        EdicaoNaBarra::Assinado => Err(ErroEditorNativo::DocumentoAssinado),
        EdicaoNaBarra::NaoPublicada | EdicaoNaBarra::ForaDaForma => {
            Err(ErroEditorNativo::LinkDoEditorNaoConferido)
        }
    }
}

fn abrir_editor(
    envio: &mut impl EnvioDoControle,
    instalacao: &Instalacao,
    alvo: &DocumentoNaArvoreAberto,
) -> Result<Editor, ErroEditorNativo> {
    conferir_oferta(
        instalacao,
        &alvo.html,
        &alvo.id_documento,
        &alvo.processo.id,
    )?;
    let tela = tela_do_documento(
        instalacao,
        &alvo.html,
        &alvo.id_documento,
        &alvo.processo.id,
    )
    .ok_or(ErroEditorNativo::TelaDoDocumentoNaoPublicada)?;
    let pagina = decodificar_resposta(envio.get(tela.as_str(), &alvo.cookie)?)?;
    assinatura(envio, instalacao, &pagina, alvo)?;
    let relativo = link_em_script(&pagina, "janelaEditor.location.href = '", "';")
        .or_else(|| link_em_script(&pagina, "var linkEditarConteudo = '", "';"))
        .ok_or(ErroEditorNativo::LinkDoEditorNaoConferido)?;
    let conferir = |chave, valor| {
        conferir_endereco_relativo_por(instalacao, &relativo, AcaoDoSei::EditorMontar, chave, valor)
            .map_err(|_| ErroEditorNativo::LinkDoEditorNaoConferido)
    };
    conferir("id_procedimento", &alvo.processo.id)?;
    let url = conferir("id_documento", &alvo.id_documento)?;
    let resposta = envio.get(&url, &alvo.cookie)?;
    let codificacao =
        codificacao_da_tela(&resposta.mime).ok_or(ErroEditorNativo::CharsetNaoSuportado)?;
    let html = decodificar_resposta(resposta)?;
    Ok(Editor {
        url,
        html,
        codificacao,
    })
}

#[cfg(test)]
pub(super) fn censo_do_classico(html: &HtmlServido) -> Option<(usize, usize)> {
    let Protocolo::Classico { versao, secoes, .. } = Protocolo::ler(html).ok()? else {
        return None;
    };
    if !matches!(versao.numero(), 1 | 2) {
        return None;
    }
    Some((
        secoes.len(),
        secoes.iter().filter(|s| s.html.is_empty()).count(),
    ))
}

#[cfg(test)]
pub(super) fn reconhece_ckeditor(html: &HtmlServido) -> bool {
    ckeditor::Configuracao::ler(html)
        .is_ok_and(|c| c.is_some_and(|c| c.versao().is_some() && c.principal().is_some()))
}

enum Protocolo {
    Classico {
        versao: VersaoDocumento,
        principal: String,
        secoes: Vec<Secao>,
    },
    Ckeditor(ckeditor::Configuracao),
}

impl Protocolo {
    fn ler(html: &HtmlServido) -> Result<Self, ErroEditorNativo> {
        if let Some(config) = ckeditor::Configuracao::ler(html)? {
            return Ok(Self::Ckeditor(config));
        }
        let forma = ErroEditorNativo::EditorForaDaForma;
        let versao = versao_do_editor(html).ok_or(ErroEditorNativo::EditorForaDaForma)?;
        let principal = principal_publicado(html).ok_or(ErroEditorNativo::EditorForaDaForma)?;
        let dom = arvore(html);
        let form = crate::formulario_web::localizar_formulario(&dom, "frmEditor")
            .map_err(ErroEditorNativo::Formulario)?;
        let secoes: Option<Vec<_>> = form
            .elementos("textarea")
            .filter(|campo| !campo.presente("disabled"))
            .map(|campo| {
                let nome = campo.atributo("name")?;
                let id = nome.strip_prefix("txaEditor_")?;
                if id.is_empty()
                    || !id.bytes().all(|b| b.is_ascii_digit())
                    || (nome == principal && campo.presente("readonly"))
                {
                    return None;
                }
                Some(Secao {
                    nome: nome.to_owned(),
                    html: secao(html, nome)?,
                })
            })
            .collect();
        let secoes = secoes.ok_or(ErroEditorNativo::EditorForaDaForma)?;
        if !secoes.iter().any(|s| s.nome == principal) {
            return Err(forma);
        }
        Ok(Self::Classico {
            versao,
            principal,
            secoes,
        })
    }

    fn versao(&self) -> Option<VersaoDocumento> {
        match self {
            Self::Classico { versao, .. } => Some(*versao),
            Self::Ckeditor(config) => config.versao(),
        }
    }

    fn principal(&self) -> Option<&str> {
        match self {
            Self::Classico { principal, .. } => Some(principal),
            Self::Ckeditor(config) => config.principal(),
        }
    }

    fn secoes(&self) -> &[Secao] {
        match self {
            Self::Classico { secoes, .. } => secoes,
            Self::Ckeditor(config) => config.secoes(),
        }
    }
}

struct AlvoDaEdicao<'a> {
    instalacao: &'a Instalacao,
    sessao: &'a SessaoDoProcesso<'a>,
    unidade: Option<&'a str>,
    documento: &'a DocumentoNaArvoreAberto,
}

/// Todas as seções retornam ao SEI no mesmo POST. A regra de LF vale para
/// cada HTML transportado, inclusive os campos não editáveis. O texto servido
/// e as cópias locais permanecem intactos; não há parser de conteúdo.
fn htmls_para_post(secoes: &[Secao], principal: &str, editado: &str) -> Vec<String> {
    secoes
        .iter()
        .map(|s| {
            if s.nome == principal {
                html_para_envio(editado)
            } else {
                html_para_envio(&s.html)
            }
        })
        .collect()
}

fn enviar_classico(
    envio: &mut impl EnvioDoControle,
    alvo: &AlvoDaEdicao<'_>,
    editor: &Editor,
    secoes: &[Secao],
    principal: &str,
    editado: &str,
) -> Result<Option<VersaoDocumento>, ErroEditorNativo> {
    let htmls = htmls_para_post(secoes, principal, editado);
    let pares: Vec<_> = secoes
        .iter()
        .zip(&htmls)
        .map(|(s, html)| (s.nome.as_str(), html.as_str()))
        .collect();
    let submissao = derivar_submissao(
        &EDITAR,
        alvo.instalacao,
        &TelaServida {
            url: &editor.url,
            html: &editor.html,
            codificacao: editor.codificacao,
        },
        alvo.unidade,
        &pares,
    )
    .map_err(ErroEditorNativo::Formulario)?;
    let consulta = crate::url_observada::UrlHttps::interpretar(submissao.url())
        .and_then(|u| u.consulta)
        .and_then(|q| ConsultaDoControlador::ler(q).ok())
        .ok_or(ErroEditorNativo::FormularioDeOutroDocumento)?;
    if consulta.unico("id_documento") != Ok(alvo.documento.id_documento.as_str())
        || consulta.unico("id_procedimento") != Ok(alvo.documento.processo.id.as_str())
    {
        return Err(ErroEditorNativo::FormularioDeOutroDocumento);
    }
    let resposta = enviar_submissao_via(submissao, alvo.sessao.cookie, |s, c| envio.post(s, c))
        .map_err(ErroEditorNativo::EnvioSemAceite)?;
    match resposta {
        RespostaDoEnvio::Html(html) => Ok(html
            .trim()
            .strip_prefix("OK ")
            .and_then(|n| VersaoDocumento::interpretar(n).ok())),
        RespostaDoEnvio::Redirecionamento(_) => Ok(None),
    }
}

#[cfg(test)]
#[path = "nativo_testes.rs"]
mod testes_releitura_condicional;

pub(super) fn executar(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    numero: &NumeroDocumento,
    previa: &PreviaCopia,
    instalacao: &Instalacao,
) -> Result<Resultado, ErroSalvarNoSei> {
    let (versao, base, edicao) = carregar_par(numero, previa)?;

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
    let protocolo = Protocolo::ler(&editor.html)?;
    if protocolo.versao() != Some(versao) {
        return Err(ErroSalvarNoSei::VersaoAlterada);
    }
    let principal = protocolo
        .principal()
        .ok_or(ErroEditorNativo::EditorForaDaForma)?;
    let editado = html_para_envio(copia_renderizada::corpo_editado(
        &base,
        &edicao,
        protocolo.secoes(),
        principal,
    )?);
    let alvo_edicao = AlvoDaEdicao {
        instalacao,
        sessao,
        unidade: cli.unidade.as_deref(),
        documento: &alvo,
    };
    let aceite = match &protocolo {
        Protocolo::Classico { secoes, .. } => enviar_classico(
            &mut envio,
            &alvo_edicao,
            &editor,
            secoes,
            principal,
            &editado,
        ),
        Protocolo::Ckeditor(config) => {
            config.salvar(instalacao, &alvo, &editor.url, principal, &editado)
        }
    };
    let nova = previa.versao_editada;
    // Erros depois da tentativa de envio podem ter persistido no SEI.
    // Sem retry nem releitura descartada, classificar o efeito como incerto.
    let aceite = match classificar_resposta_do_post(aceite)? {
        RespostaDoPost::Aceite(versao) => versao,
        RespostaDoPost::Incerta => return Ok(Resultado::Incerto),
    };
    // GET de assinatura/editor só após resposta com a versão esperada.
    let persistido = reabrir_apos_aceite(Ok(aceite), nova, || {
        abrir_editor(&mut envio, instalacao, &alvo)
    })?;
    let Some(Ok(persistido)) = persistido else {
        return Ok(Resultado::Incerto);
    };
    let Ok(persistido) = Protocolo::ler(&persistido.html) else {
        return Ok(Resultado::Incerto);
    };
    if !releitura::confere(&protocolo, &persistido, nova, &editado) {
        // O POST pode ter gravado outra versão/conteúdo; não sugerir retry.
        return Ok(Resultado::Incerto);
    }
    Ok(Resultado::Confirmado(nova))
}
