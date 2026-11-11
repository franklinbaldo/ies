//! BDD-89-02/03/04: editor SEI 4 projetado a partir de observações, releitura.
//! A escrita externa permanece bloqueada até validação da instalação.
use crate::Cli;
use crate::acao_do_sei::AcaoDoSei;
use crate::arquivo_copia::ler_regular;
use crate::copia_documento::{self, ErroCopia, PreviaCopia};
use crate::documento::NumeroDocumento;
use crate::documento_versoes::{ErroVersao, VersaoDocumento};
use crate::formulario_web::{
    CodificacaoDaTela, ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, derivar_submissao,
    enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::processo_concluir_cli::com_sessao;
use crate::processo_obter::{
    ErroDocumentoObter, PedidoDeVersoes, PedidoDoAnexo, PedidoDoDocumento, SessaoDoProcesso,
    obter_documento_com_propriedades,
};
use crate::transporte::decodificar_resposta;
use crate::visao_do_controle::{EnvioDoControle as _, EnvioHttps};
use serde_json::json;
use std::path::Path;

/// Fluxo SEI 4 observado no catálogo (UnB/PEN1/UFRJ), ainda sem replay
/// dos corpos originais de POST. Só usado na instalação sintética sei.test.
const EDITAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::EditorSalvar,
    formulario: "frmEditor",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnVersao", "hdnIgnorarNovaVersao"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["txaEditor_1", "confirmar_assinaturas"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::DocumentoEditado,
    },
};

/// O corpo é uma região explícita da resposta de teste, não um parser de SEI.
fn partes_da_foto(html: &str) -> Option<(&str, &str, &str)> {
    let (inicio, resto) = html.split_once("<main id='corpoEditavel'>")?;
    let (corpo, fim) = resto.split_once("</main>")?;
    Some((inicio, corpo, fim))
}

fn corpo_sintetico(html: &str) -> Option<&str> {
    partes_da_foto(html).map(|(_, corpo, _)| corpo)
}

/// Um fato por erro, com a causa; nada foi gravado, salvo em
/// `EdicaoNaoComprovada`.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum ErroSalvarNoSei {
    Local(#[source] ErroCopia),
    PastaAtualIlegivel(#[source] std::io::Error),
    /// Só a instalação sintética (`sei.test`) tem o editor observado.
    InstalacaoSemEditorObservado,
    LeituraDoDocumento(#[source] ErroDocumentoObter),
    Cookie(#[source] crate::transporte::FalhaTransporte),
    VersaoAlterada,
    BaseAlterada,
    /// A base não é UTF-8; guarda a causa da conversão.
    BaseNaoUtf8(#[source] std::str::Utf8Error),
    /// O número da versão seguinte não é válido; guarda a causa.
    ProximaVersaoInvalida(#[source] ErroVersao),
    /// O número da versão base não é válido; guarda a causa.
    VersaoBaseInvalida(#[source] ErroVersao),
    FotoLocalForaDaForma,
    EditorNaoServido(#[source] crate::transporte::FalhaTransporte),
    EditorForaDaForma,
    Formulario(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    RegiaoNaoEditavelAlterada,
    /// O POST saiu e a releitura não comprovou a edição: não repita
    /// `--confirmar` sem releitura independente.
    EdicaoNaoComprovada,
}

fn ler_par(
    pasta: &Path,
    numero: &NumeroDocumento,
    n: VersaoDocumento,
) -> Result<(String, String), ErroSalvarNoSei> {
    copia_documento::pasta_segura(pasta).map_err(ErroSalvarNoSei::Local)?;
    let seguinte =
        VersaoDocumento::nova(n.numero() + 1).map_err(ErroSalvarNoSei::ProximaVersaoInvalida)?;
    let ler = |v, ausente| {
        ler_regular(
            &pasta.join(crate::foto_documento::nome(numero.as_str(), v)),
            ausente,
        )
        .map_err(ErroSalvarNoSei::Local)
        .and_then(|bytes| {
            String::from_utf8(bytes).map_err(|erro| ErroSalvarNoSei::BaseNaoUtf8(erro.utf8_error()))
        })
    };
    Ok((
        ler(n, ErroCopia::BaseVersaoAusente)?,
        ler(seguinte, ErroCopia::VersaoEditadaAusente)?,
    ))
}

enum Resultado {
    Recusado(ErroSalvarNoSei),
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
    html.contains(&format!("name='hdnVersao' value='{versao_base}'"))
        && html.contains("name='hdnIgnorarNovaVersao' value='N'")
        && html.contains("name='txaEditor_1'")
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
) -> Result<(VersaoDocumento, String, String), ErroSalvarNoSei> {
    let n =
        VersaoDocumento::nova(previa.versao_base).map_err(ErroSalvarNoSei::VersaoBaseInvalida)?;
    let pasta = std::env::current_dir().map_err(ErroSalvarNoSei::PastaAtualIlegivel)?;
    let (base, editada) = ler_par(&pasta, numero, n)?;
    Ok((n, base, editada))
}

/// A estrutura da foto local distingue os trechos editáveis da moldura SEI.
fn corpos_da_edicao<'a>(
    base: &'a str,
    editada: &'a str,
) -> Result<(&'a str, &'a str), ErroSalvarNoSei> {
    let (inicio_base, corpo_base, fim_base) =
        partes_da_foto(base).ok_or(ErroSalvarNoSei::FotoLocalForaDaForma)?;
    let (inicio_edicao, corpo_editado, fim_edicao) =
        partes_da_foto(editada).ok_or(ErroSalvarNoSei::FotoLocalForaDaForma)?;
    if inicio_base != inicio_edicao || fim_base != fim_edicao {
        return Err(ErroSalvarNoSei::RegiaoNaoEditavelAlterada);
    }
    if corpo_base == corpo_editado {
        return Err(ErroSalvarNoSei::BaseAlterada);
    }
    Ok((corpo_base, corpo_editado))
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
        _ => return Resultado::Recusado(ErroSalvarNoSei::InstalacaoSemEditorObservado),
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
    let lido = match obter_documento_com_propriedades(
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
    ) {
        Ok(lido) => lido,
        Err(erro) => return Resultado::Recusado(ErroSalvarNoSei::LeituraDoDocumento(erro)),
    };
    if lido.versoes.atual.as_ref().map(|v| v.numero()) != Ok(n.numero()) {
        return Resultado::Recusado(ErroSalvarNoSei::VersaoAlterada);
    }
    if lido.versoes.foto.as_ref().map(|foto| foto.html.as_str()) != Some(local_base.as_str()) {
        return Resultado::Recusado(ErroSalvarNoSei::BaseAlterada);
    }
    let (_base, editada) = match corpos_da_edicao(&local_base, &local_editada) {
        Ok(par) => par,
        Err(falha) => return Resultado::Recusado(falha),
    };
    let url = format!(
        "{}/controlador.php?acao=editor_montar&id_documento={}",
        sessao.origem, lido.id_documento
    );
    let cookie = match sessao.cookie.valor() {
        Ok(cookie) => cookie,
        Err(falha) => return Resultado::Recusado(ErroSalvarNoSei::Cookie(falha)),
    };
    let editor = match envio.get(&url, cookie).and_then(decodificar_resposta) {
        Ok(editor) => editor,
        Err(falha) => return Resultado::Recusado(ErroSalvarNoSei::EditorNaoServido(falha)),
    };
    if !editor_observado(&editor, n.numero()) {
        return Resultado::Recusado(ErroSalvarNoSei::EditorForaDaForma);
    }
    let envio_unico = derivar_submissao(
        &EDITAR,
        &instalacao,
        &TelaServida {
            url: &url,
            html: &editor,
            codificacao: CodificacaoDaTela::Utf8,
        },
        cli.unidade.as_deref(),
        &[("txaEditor_1", editada), ("confirmar_assinaturas", "1")],
    );
    let envio_unico = match envio_unico {
        Ok(envio_unico) => envio_unico,
        Err(erro) => return Resultado::Recusado(ErroSalvarNoSei::Formulario(erro)),
    };
    // Daqui em diante até uma falha de conexão pode ter produzido efeito.
    let post = enviar_submissao_via(envio_unico, sessao.cookie, |s, c| envio.post(s, c));
    let respondeu = matches!(
        post,
        Ok(RespostaDoEnvio::Html(ref resposta))
            if resposta.trim() == format!("OK {}", previa.versao_editada)
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

pub(super) fn confirmar(
    cli: &Cli,
    numero: &NumeroDocumento,
    previa: &PreviaCopia,
) -> anyhow::Result<()> {
    com_sessao(cli, |sessao, indice| {
        match executar(cli, sessao, indice, numero, previa) {
            Resultado::Confirmado(versao) => {
                if cli.json {
                    let mut valor = json!({
                        "documento": numero.as_str(),
                        "confirmado": true,
                        "versao_criada": versao,
                        "releitura_confirmada": true,
                        "assinaturas_apos": 0
                    });
                    crate::indice_local::anexar_avisos(&mut valor, indice);
                    println!("{valor}");
                } else {
                    println!(
                        "IES: documento {} salvo na versão {versao}, conferida por releitura.",
                        numero.as_str()
                    );
                }
                Ok(())
            }
            Resultado::Recusado(falha) => Err(falha.into()),
            Resultado::Incerto => Err(ErroSalvarNoSei::EdicaoNaoComprovada.into()),
        }
    })
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
            assert!(corpos_da_edicao(BASE, &edicao).is_err());
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

    #[cfg(unix)]
    #[test]
    fn base_utf8_invalida_preserva_causa_sem_expor_conteudo_privado() {
        use super::{ErroSalvarNoSei, ler_par};
        use crate::documento::NumeroDocumento;
        use crate::documento_versoes::VersaoDocumento;
        use std::error::Error;
        use std::os::unix::fs::PermissionsExt;
        use std::sync::atomic::{AtomicUsize, Ordering};

        static PROXIMA: AtomicUsize = AtomicUsize::new(0);
        let pasta = std::env::temp_dir().join(format!(
            "ies-utf8-base-{}-{}",
            std::process::id(),
            PROXIMA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&pasta).unwrap();
        let numero = NumeroDocumento::novo("719").unwrap();
        let arquivo = pasta.join("719.v1.html");
        let sigilo = b"SIGILO_MINUTA_719_\xff";
        std::fs::write(&arquivo, sigilo).unwrap();
        std::fs::set_permissions(&arquivo, std::fs::Permissions::from_mode(0o600)).unwrap();

        let erro = ler_par(&pasta, &numero, VersaoDocumento::nova(1).unwrap()).unwrap_err();
        std::fs::remove_file(&arquivo).unwrap();
        std::fs::remove_dir(&pasta).unwrap();

        assert!(matches!(erro, ErroSalvarNoSei::BaseNaoUtf8(_)));
        let fonte = erro.source().expect("fonte do UTF-8");
        assert!(
            fonte.is::<std::str::Utf8Error>(),
            "a causa não deve reter o Vec privado"
        );
        let exibido = format!("{erro:?}\n{erro}\n{fonte:?}\n{fonte}");
        assert!(
            !exibido.contains(&format!("{sigilo:?}")) && !exibido.contains("SIGILO_MINUTA_719_"),
            "nenhum formato público do erro deve conter bytes da minuta"
        );
    }
}
