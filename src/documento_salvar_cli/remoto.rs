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
    let Ok(n) = VersaoDocumento::nova(previa.versao_base) else {
        return Resultado::Recusado(Falha::VersaoAlterada);
    };
    let Ok(pasta) = std::env::current_dir() else {
        return Resultado::Recusado(Falha::Leitura);
    };
    let (local_base, local_editada) = match ler_par(&pasta, numero, n) {
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
    if lido
        .versoes
        .foto
        .as_ref()
        .is_none_or(|foto| foto.html != local_base)
    {
        return Resultado::Recusado(Falha::BaseAlterada);
    }
    let (Some(base), Some(editada)) = (
        corpo_sintetico(&local_base),
        corpo_sintetico(&local_editada),
    ) else {
        return Resultado::Recusado(Falha::EditorIndisponivel);
    };
    if base == editada {
        return Resultado::Recusado(Falha::BaseAlterada);
    }
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
    if !editor.contains(&format!("name='versao_base' value='{}'", n.numero())) {
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
    let versao_nova = previa.versao_editada;
    let url_verificacao = format!(
        "{}/controlador.php?acao=documento_visualizar&id_documento={}&versao={versao_nova}",
        sessao.origem, lido.id_documento
    );
    let releitura = envio
        .get(&url_verificacao, cookie)
        .and_then(decodificar_resposta);
    let Ok(html) = releitura else {
        return Resultado::Incerto;
    };
    let respondeu = matches!(
        post,
        Ok(RespostaDoEnvio::Redirecionamento(ref r))
            if r.localizacao().is_some_and(|destino|
                destino.contains("acao=documento_visualizar")
                && destino.contains(&format!("id_documento={}", lido.id_documento))
                && destino.contains(&format!("versao={versao_nova}")))
    );
    let versao_igual = crate::rodape_do_documento::versao_do_rodape(&html)
        .is_some_and(|v| v.numero() == versao_nova);
    let identidade_igual = crate::documento_web::identidade_do_titulo(&html)
        .is_ok_and(|i| i.numero == numero.as_str());
    if respondeu && versao_igual && identidade_igual && corpo_sintetico(&html) == Some(editada) {
        Resultado::Confirmado(versao_nova)
    } else {
        Resultado::Incerto
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
                            "releitura_confirmada": true
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
            Resultado::Recusado(falha) => {
                let _ = match &falha {
                    Falha::Local(causa) => Some(causa),
                    _ => None,
                };
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
