//! Confirmação sintética BDD-89-02/03 do editor: uma escrita, releitura.
//! A escrita externa permanece bloqueada até validação da instalação.
use crate::arquivo_copia::ler_regular;
use crate::copia_documento::{self, ErroCopia, PreviaCopia};
use crate::documento::NumeroDocumento;
use crate::documento_versoes::VersaoDocumento;
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    CodificacaoDaTela, ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, derivar_submissao,
    enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::processo_concluir_cli::com_sessao;
use crate::processo_obter::{
    PedidoDeVersoes, PedidoDoAnexo, PedidoDoDocumento, SessaoDoProcesso,
    obter_documento_com_propriedades,
};
use crate::transporte::decodificar_resposta;
use crate::visao_do_controle::{EnvioDoControle as _, EnvioHttps};
use crate::{Cli, erro_fonte};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

/// Operação local simulada; o POST usa os controles publicados na tela.
/// Não é contrato do HTML real do SEI: a hipótese só vale no anti-IES.
const EDITAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "documento_editar",
    formulario: "frmDocumentoEditar",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["versao_base"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["html_base", "html_editado", "confirmar_assinaturas"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::DocumentoEditado,
    },
};

/// O corpo é uma região explícita da resposta de teste, não um parser de SEI.
fn corpo_sintetico(html: &str) -> Option<&str> {
    html.split_once("<main id='corpoEditavel'>")?
        .1
        .split_once("</main>")
        .map(|(corpo, _)| corpo)
}

enum Falha {
    Local(ErroCopia),
    Leitura,
    VersaoAlterada,
    BaseAlterada,
    EditorIndisponivel,
    Formulario,
}

impl Falha {
    fn mensagem(&self) -> &'static str {
        match self {
            Self::Local(_) => "arquivos locais não puderam ser relidos com segurança",
            Self::Leitura => "versão remota não pôde ser conferida",
            Self::VersaoAlterada => "a versão remota atual diverge da base local",
            Self::BaseAlterada => "a base local não corresponde ao HTML remoto da versão",
            Self::EditorIndisponivel => "a instalação sintética não ofereceu editor verificável",
            Self::Formulario => "o formulário publicado não corresponde ao editor simulado",
        }
    }
}

fn ler_par(
    pasta: &Path,
    numero: &NumeroDocumento,
    n: VersaoDocumento,
) -> Result<(String, String), Falha> {
    copia_documento::pasta_segura(pasta).map_err(Falha::Local)?;
    let seguinte = VersaoDocumento::nova(n.numero() + 1).map_err(|_| Falha::VersaoAlterada)?;
    let ler = |v, ausente| {
        ler_regular(
            &pasta.join(crate::foto_documento::nome(numero.as_str(), v)),
            ErroCopia::Html(ErroHtml::ausente(ausente)),
        )
        .map_err(Falha::Local)
        .and_then(|bytes| String::from_utf8(bytes).map_err(|_| Falha::BaseAlterada))
    };
    Ok((
        ler(n, "base_versao_ausente")?,
        ler(seguinte, "versao_editada_ausente")?,
    ))
}

enum Resultado {
    Recusado(Falha),
    Incerto,
    Confirmado(u32),
}

/// A confirmação requer uma releitura independente da nova versão,
/// além da resposta ao único POST. Sem essa prova, o efeito é incerto.
/// A contagem é informação explícita no editor sintético, não heurística
/// sobre a presença de blocos de assinatura no HTML da versão.
fn assinaturas_no_editor(html: &str) -> Option<u32> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let parser = dom.parser();
    let mut contagem = None;
    for handle in dom.query_selector("output")? {
        let node = handle.get(parser)?;
        let tag = node.as_tag()?;
        let attrs = crate::html_atributos::atributos(tag).ok()?;
        if attrs.valor("id").ok()? == Some("assinaturasAtuais") {
            if contagem.is_some() {
                return None;
            }
            contagem = Some(node.inner_text(parser).parse().ok()?);
        }
    }
    contagem
}

fn editor_observado(html: &str, versao_base: u32) -> bool {
    html.contains(&format!("name='versao_base' value='{versao_base}'"))
        && assinaturas_no_editor(html).is_some()
}

struct ConferenciaPublicacao<'a> {
    origem: &'a str,
    cookie: &'a str,
    id_documento: &'a str,
    url_editor: &'a str,
    numero: &'a NumeroDocumento,
    versao_nova: u32,
    editada: &'a str,
    respondeu: bool,
}

fn verificar_publicacao(envio: &mut EnvioHttps, dados: &ConferenciaPublicacao<'_>) -> Resultado {
    let &ConferenciaPublicacao {
        origem,
        cookie,
        id_documento,
        url_editor,
        numero,
        versao_nova,
        editada,
        respondeu,
    } = dados;
    let url = format!(
        "{origem}/controlador.php?acao=documento_visualizar&id_documento={id_documento}&versao={versao_nova}"
    );
    let Ok(html) = envio.get(&url, cookie).and_then(decodificar_resposta) else {
        return Resultado::Incerto;
    };
    // Um segundo GET do editor confirma o estado das assinaturas após o POST.
    let assinaturas_conferidas = envio
        .get(url_editor, cookie)
        .and_then(decodificar_resposta)
        .ok()
        .and_then(|pagina| assinaturas_no_editor(&pagina))
        == Some(0);
    let versao_igual = crate::rodape_do_documento::versao_do_rodape(&html)
        .is_some_and(|v| v.numero() == versao_nova);
    let identidade_igual = crate::documento_web::identidade_do_titulo(&html)
        .is_ok_and(|i| i.numero == numero.as_str());
    if respondeu
        && versao_igual
        && identidade_igual
        && assinaturas_conferidas
        && corpo_sintetico(&html) == Some(editada)
    {
        Resultado::Confirmado(versao_nova)
    } else {
        Resultado::Incerto
    }
}

fn carregar_par(
    numero: &NumeroDocumento,
    previa: &PreviaCopia,
) -> Result<(VersaoDocumento, String, String), Falha> {
    let n = VersaoDocumento::nova(previa.versao_base).map_err(|_| Falha::VersaoAlterada)?;
    let pasta = std::env::current_dir().map_err(|_| Falha::Leitura)?;
    let (base, editada) = ler_par(&pasta, numero, n)?;
    Ok((n, base, editada))
}

/// A estrutura da foto local distingue os trechos editáveis da moldura SEI.
fn corpos_da_edicao<'a>(base: &'a str, editada: &'a str) -> Result<(&'a str, &'a str), Falha> {
    let base = corpo_sintetico(base).ok_or(Falha::EditorIndisponivel)?;
    let editada = corpo_sintetico(editada).ok_or(Falha::EditorIndisponivel)?;
    if base == editada {
        return Err(Falha::BaseAlterada);
    }
    Ok((base, editada))
}

fn executar(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &crate::indice_local::IndiceLocal,
    numero: &NumeroDocumento,
    previa: &PreviaCopia,
) -> Resultado {
    let instalacao = match Instalacao::interpretar(sessao.origem) {
        Some(instalacao) if instalacao.host() == "sei.test" => instalacao,
        _ => return Resultado::Recusado(Falha::EditorIndisponivel),
    };
    let (n, local_base, local_editada) = match carregar_par(numero, previa) {
        Ok(par) => par,
        Err(falha) => return Resultado::Recusado(falha),
    };
    let mut envio = EnvioHttps;
    let pedido = PedidoDoDocumento {
        numero,
        unidade: cli.unidade.as_deref(),
        processo: None,
        anexo: PedidoDoAnexo::default(),
    };
    let Ok(lido) = obter_documento_com_propriedades(
        sessao,
        &pedido,
        PedidoDeVersoes {
            versao: Some(n),
            foto: true,
            cache: false,
        },
        indice,
        &mut envio,
        false,
    ) else {
        return Resultado::Recusado(Falha::Leitura);
    };
    if lido.versoes.atual.as_ref().map(|v| v.numero()) != Ok(n.numero()) {
        return Resultado::Recusado(Falha::VersaoAlterada);
    }
    if lido.versoes.foto.as_ref().map(|foto| foto.html.as_str()) != Some(local_base.as_str()) {
        return Resultado::Recusado(Falha::BaseAlterada);
    }
    let (base, editada) = match corpos_da_edicao(&local_base, &local_editada) {
        Ok(par) => par,
        Err(falha) => return Resultado::Recusado(falha),
    };
    let url = format!(
        "{}/controlador.php?acao=documento_editar&id_documento={}",
        sessao.origem, lido.id_documento
    );
    let Ok(cookie) = sessao.cookie.valor() else {
        return Resultado::Recusado(Falha::Leitura);
    };
    let Ok(editor) = envio.get(&url, cookie).and_then(decodificar_resposta) else {
        return Resultado::Recusado(Falha::EditorIndisponivel);
    };
    if !editor_observado(&editor, n.numero()) {
        return Resultado::Recusado(Falha::Formulario);
    }
    let envio_unico = derivar_submissao(
        &EDITAR,
        &instalacao,
        &TelaServida {
            url: &url,
            html: &editor,
            codificacao: CodificacaoDaTela::Utf8,
        },
        cli.unidade.as_deref().unwrap_or(""),
        &[
            ("html_base", base),
            ("html_editado", editada),
            ("confirmar_assinaturas", "1"),
        ],
    );
    let Ok(envio_unico) = envio_unico else {
        return Resultado::Recusado(Falha::Formulario);
    };
    // Daqui em diante até uma falha de conexão pode ter produzido efeito.
    let post = enviar_submissao_via(envio_unico, sessao.cookie, |s, c| envio.post(s, c));
    let respondeu = matches!(
        post,
        Ok(RespostaDoEnvio::Redirecionamento(ref r))
            if r.localizacao().is_some_and(|destino|
                destino.contains("acao=documento_visualizar")
                && destino.contains(&format!("id_documento={}", lido.id_documento))
                && destino.contains(&format!("versao={}", previa.versao_editada)))
    );
    verificar_publicacao(
        &mut envio,
        &ConferenciaPublicacao {
            origem: sessao.origem,
            cookie,
            id_documento: &lido.id_documento,
            url_editor: &url,
            numero,
            versao_nova: previa.versao_editada,
            editada,
            respondeu,
        },
    )
}


#[cfg(test)]
mod testes_moldura {
    use super::corpos_da_edicao;

    const BASE: &str = "<html><head><title>Documento</title></head><body><main id='corpoEditavel'><p>Antes</p></main><div>Versão 3</div></body></html>";

    #[test]
    fn nao_publica_uma_edicao_que_tambem_mudou_a_moldura_nao_editavel() {
        let corpo_novo = BASE.replace("<p>Antes</p>", "<p>Depois</p>");
        for edicao in [
            corpo_novo.replace("<title>Documento</title>", "<title>Outro</title>"),
            corpo_novo.replace("<div>Versão 3</div>", "<div>Versão 99</div>"),
        ] {
            assert!(matches!(
                corpos_da_edicao(BASE, &edicao),
                Err(_)
            ));
        }
    }

    #[test]
    fn mantem_o_caminho_editavel_quando_so_o_corpo_muda() {
        let edicao = BASE.replace("<p>Antes</p>", "<p>Depois</p>");
        assert!(matches!(
            corpos_da_edicao(BASE, &edicao),
            Ok(("<p>Antes</p>", "<p>Depois</p>"))
        ));
    }
}

pub(super) fn confirmar(cli: &Cli, numero: &NumeroDocumento, previa: &PreviaCopia) -> ExitCode {
    com_sessao(cli, |sessao, indice| {
        match executar(cli, sessao, indice, numero, previa) {
            Resultado::Confirmado(versao) => {
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "documento": numero.as_str(),
                            "confirmado": true,
                            "versao_criada": versao,
                            "releitura_confirmada": true,
                            "assinaturas_apos": 0
                        })
                    );
                } else {
                    println!(
                        "IES: documento {} salvo na versão {versao}, conferida por releitura.",
                        numero.as_str()
                    );
                }
                ExitCode::SUCCESS
            }
            Resultado::Recusado(Falha::Local(causa)) => {
                erro_fonte(cli.json, causa.subcodigo(), Falha::Local(causa).mensagem())
            }
            Resultado::Recusado(falha) => {
                erro_fonte(cli.json, "escrita_remota_nao_validada", falha.mensagem())
            }
            Resultado::Incerto => {
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "codigo": "efeito_incerto",
                            "erro": "edição não comprovada; não repita --confirmar sem releitura independente",
                            "confirmado": false,
                            "retentavel": false
                        })
                    );
                } else {
                    eprintln!("IES: edição não comprovada; não repita --confirmar sem releitura.");
                }
                ExitCode::from(3)
            }
        }
    })
}
