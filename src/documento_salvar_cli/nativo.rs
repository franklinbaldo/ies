//! Salvamento no editor publicado para uma minuta sem assinatura.
use super::{ErroSalvarNoSei, Resultado, carregar_par, html_para_envio};
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

#[path = "classico.rs"]
mod classico;
use classico::{principal_publicado, secao, versao_do_editor};

#[path = "ckeditor.rs"]
mod ckeditor;
#[path = "copia_renderizada.rs"]
mod copia_renderizada;
use copia_renderizada::Secao;

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
    ConteudoPersistidoDiferente { versao: u32 },
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

fn abrir_editor(
    envio: &mut EnvioHttps,
    instalacao: &Instalacao,
    alvo: &DocumentoNaArvoreAberto,
) -> Result<Editor, ErroEditorNativo> {
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
pub(super) fn reconhece_classico(html: &HtmlServido) -> bool {
    versao_do_editor(html).is_some_and(|v| matches!(v.numero(), 1 | 2))
        && principal_publicado(html).is_some()
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

fn enviar_classico(
    envio: &mut EnvioHttps,
    alvo: &AlvoDaEdicao<'_>,
    editor: &Editor,
    secoes: &[Secao],
    principal: &str,
    editado: &str,
) -> Result<Option<VersaoDocumento>, ErroEditorNativo> {
    let pares: Vec<_> = secoes
        .iter()
        .map(|s| {
            (
                s.nome.as_str(),
                if s.nome == principal {
                    editado
                } else {
                    s.html.as_str()
                },
            )
        })
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

/// Releitura do editor pode emitir vários GETs (inclusive assinatura).
/// Só a realiza quando o POST recebeu aceite da versão esperada.
/// Um erro ou aceite inconclusivo não justifica GET descartado.
fn reabrir_apos_aceite<T>(
    aceite: Result<Option<VersaoDocumento>, ErroEditorNativo>,
    esperada: u32,
    reler: impl FnOnce() -> T,
) -> Result<Option<T>, ErroEditorNativo> {
    if aceite?.map(VersaoDocumento::numero) != Some(esperada) {
        return Ok(None);
    }
    Ok(Some(reler()))
}

#[cfg(test)]
mod testes_releitura_condicional {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn erro_ou_aceite_inconclusivo_nao_faz_releitura() {
        let leituras = Cell::new(0usize);
        let reler = || {
            leituras.set(leituras.get() + 1);
            42
        };
        assert!(matches!(
            reabrir_apos_aceite(Err(ErroEditorNativo::EditorForaDaForma), 2, reler),
            Err(ErroEditorNativo::EditorForaDaForma)
        ));
        assert_eq!(leituras.get(), 0);

        let sem_versao = reabrir_apos_aceite(Ok(None), 2, reler).unwrap();
        assert_eq!(sem_versao, None);
        let errada = reabrir_apos_aceite(
            Ok(Some(VersaoDocumento::nova(3).unwrap())),
            2,
            reler,
        ).unwrap();
        assert_eq!(errada, None);
        assert_eq!(leituras.get(), 0);

        let correta = reabrir_apos_aceite(
            Ok(Some(VersaoDocumento::nova(2).unwrap())),
            2,
            reler,
        ).unwrap();
        assert_eq!(correta, Some(42));
        assert_eq!(leituras.get(), 1);
    }
}

pub(super) fn executar(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    numero: &NumeroDocumento,
    previa: &PreviaCopia,
    instalacao: &Instalacao,
    normalizar: bool,
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
    let editado = html_para_envio(
        copia_renderizada::corpo_editado(&base, &edicao, protocolo.secoes(), principal)?,
        normalizar,
    )?;
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
    // Não emitir GET de assinatura/editor quando o POST falhou, não recebeu
    // versão ou recebeu uma versão inesperada: a leitura seria descartada.
    let persistido = reabrir_apos_aceite(aceite, nova, || {
        abrir_editor(&mut envio, instalacao, &alvo)
    })?;
    let Some(Ok(persistido)) = persistido else {
        return Ok(Resultado::Incerto);
    };
    let Ok(persistido) = Protocolo::ler(&persistido.html) else {
        return Ok(Resultado::Incerto);
    };
    if persistido.versao().map(VersaoDocumento::numero) != Some(nova)
        || persistido.principal() != Some(principal)
    {
        return Ok(Resultado::Incerto);
    }
    if !persistido
        .secoes()
        .iter()
        .any(|s| s.nome == principal && s.html == editado.as_ref())
    {
        return Err(ErroEditorNativo::ConteudoPersistidoDiferente { versao: nova }.into());
    }
    Ok(Resultado::Confirmado(nova))
}
