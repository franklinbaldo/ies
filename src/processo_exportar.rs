//! Geração nativa do processo inteiro (#889). Não executa JavaScript,
//! não fabrica links e não repete a geração.
use crate::acao_do_sei::AcaoDoSei;
use crate::anexo_documento::LimiteDoAnexo;
use crate::arquivo_privado::{Parcial, Politica, conferir_diretorios};
use crate::arvore_processo::conferir_endereco_relativo;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, OperacaoDeFormulario, RespostaDoEnvio,
    ResultadoDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao, enviar_submissao_via,
    localizar_formulario,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, leitura, obter_tela};
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso, abrir_para_escrever};
use crate::tela_html::{HtmlServido, arvore};
use crate::transporte::resposta_binaria::RespostaBinaria;
use crate::visao_do_controle::{EnvioDoControle as _, EnvioHttps};
use crate::{Cli, Domain, Processo, ProcessoAcao};
use anyhow::{Context as _, bail, ensure};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

#[derive(Clone, Copy, Debug, clap::ValueEnum, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Formato {
    Pdf,
    Zip,
}

impl Formato {
    fn operacao(self) -> OperacaoDeFormulario {
        let (acao, formulario) = match self {
            Self::Pdf => (AcaoDoSei::ProcedimentoGerarPdf, "frmProcedimentoPdf"),
            Self::Zip => (AcaoDoSei::ProcedimentoGerarZip, "frmProcedimentoZip"),
        };
        OperacaoDeFormulario {
            acao,
            formulario,
            envio: EnvioDoFormulario::SubmitPorScript,
            campos_ocultos_exigidos: &["hdnFlagGerar"],
            campos_ocultos_reescritos: &["hdnFlagGerar"],
            campos_usuario: &["rdoTipo"],
            resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
            unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
            efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
                evidencia: "UnB: Todos caabe699, gerar_arquivo; sem escrita cadastral observada",
            },
        }
    }

    fn mime(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Zip => "application/zip",
        }
    }

    fn assinatura(self, bytes: &[u8]) -> bool {
        match self {
            Self::Pdf => bytes.starts_with(b"%PDF-"),
            Self::Zip => bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06"),
        }
    }
}

fn submissao(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    id: &str,
    formato: Formato,
    unidade: Option<&str>,
) -> anyhow::Result<SubmissaoDerivada> {
    let operacao = formato.operacao();
    let dom = arvore(tela.html);
    let form = localizar_formulario(&dom, operacao.formulario)?;
    let action = form
        .atributo("action")
        .context("ActionDeExportacaoAusente")?;
    conferir_endereco_relativo(instalacao, action, operacao.acao, id)?;
    let todos = form
        .descendentes()
        .filter(|e| {
            e.nome() == "input"
                && e.atributo("type") == Some("radio")
                && e.atributo("name") == Some("rdoTipo")
                && e.atributo("value") == Some("T")
                && !e.presente("disabled")
        })
        .count();
    ensure!(todos == 1, "OpcaoTodosNaoPublicadaUmaVez");
    Ok(derivar_submissao(
        &operacao,
        instalacao,
        tela,
        unidade,
        &[("rdoTipo", "T"), ("hdnFlagGerar", "1")],
    )?)
}

/// Forma literal observada em `gerar_pdf_resposta.html`; nunca executa script.
fn endereco_download(
    instalacao: &Instalacao,
    html: &HtmlServido,
    id: &str,
) -> anyhow::Result<String> {
    let dom = arvore(html);
    ensure!(
        dom.elementos("iframe")
            .filter(|e| e.atributo("id") == Some("ifrDownload"))
            .count()
            == 1,
        "IframeDownloadAusenteOuRepetido"
    );
    let mut urls = Vec::new();
    for script in dom.elementos("script").filter(|e| !e.presente("src")) {
        let texto = script.texto();
        for linha in texto.lines() {
            if let Some(url) = linha
                .trim()
                .strip_prefix("document.getElementById('ifrDownload').src = '")
                .and_then(|linha| linha.strip_suffix("';"))
            {
                urls.push(url.to_owned());
            }
        }
    }
    let [url] = urls.as_slice() else {
        bail!("DownloadNaoPublicadoUmaVez");
    };
    Ok(conferir_endereco_relativo(
        instalacao,
        url,
        AcaoDoSei::ExibirArquivo,
        id,
    )?)
}

#[derive(serde::Serialize)]
struct Arquivo {
    tamanho_bytes: u64,
    sha256: String,
}

fn gravar<R: Read>(
    mut resposta: RespostaBinaria<R>,
    formato: Formato,
    saida: &Path,
    limite: LimiteDoAnexo,
) -> anyhow::Result<Arquivo> {
    ensure!(resposta.status == 200, "DownloadNaoRetornou200");
    let tipos = resposta.valores("content-type");
    ensure!(
        matches!(tipos.as_slice(), [tipo] if tipo.split(';').next().is_some_and(|t| t.trim().eq_ignore_ascii_case(formato.mime()))),
        "TipoDoDownloadDivergente"
    );
    let politica = Politica::exclusiva()
        .diretorio_do_usuario()
        .com_limite(limite.bytes());
    let mut pdf = [0u8; 5];
    let mut zip = [0u8; 4];
    let prefixo: &mut [u8] = match formato {
        Formato::Pdf => &mut pdf,
        Formato::Zip => &mut zip,
    };
    resposta.corpo.read_exact(prefixo)?;
    ensure!(
        formato.assinatura(prefixo),
        "AssinaturaDoDownloadDivergente"
    );
    let mut parcial = Parcial::abrir(saida, politica)?;
    let mut hash = Sha256::new();
    parcial.escrever(prefixo)?;
    let mut tamanho = u64::try_from(prefixo.len())?;
    hash.update(prefixo);
    let mut bloco = [0u8; 16 * 1024];
    loop {
        let lido = resposta.corpo.read(&mut bloco)?;
        if lido == 0 {
            break;
        }
        let bytes = bloco.get(..lido).context("LeituraAcimaDoBloco")?;
        parcial.escrever(bytes)?;
        hash.update(bytes);
        tamanho += u64::try_from(lido)?;
    }
    let comprimentos = resposta.valores("content-length");
    ensure!(
        comprimentos.is_empty()
            || matches!(comprimentos.as_slice(), [c] if c.parse::<u64>().ok() == Some(tamanho)),
        "ComprimentoDoDownloadDivergente"
    );
    parcial.confirmar()?;
    Ok(Arquivo {
        tamanho_bytes: tamanho,
        sha256: hex::encode(hash.finalize()),
    })
}

struct Pedido<'a> {
    numero: &'a NumeroProcesso,
    unidade: Option<&'a str>,
    formato: Formato,
    saida: &'a Path,
    limite: LimiteDoAnexo,
}

fn exportar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &Pedido<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> anyhow::Result<(Arquivo, u8)> {
    let instalacao = Instalacao::interpretar(sessao.origem).context("InstalacaoInvalida")?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.numero, pedido.unidade, true),
        indice,
        &mut rede,
    )?;
    let id = aberto.arvore.processo.id;
    let link = match pedido.formato {
        Formato::Pdf => acoes.exportacao.pdf,
        Formato::Zip => acoes.exportacao.zip,
    }?
    .context("ExportacaoNaoOferecida")?;
    let cookie = acoes.cookie;
    let (html, mime) = obter_tela(&mut rede, &cookie, link.as_str())?;
    let codificacao =
        crate::pagina_do_controle::codificacao_da_tela(&mime).context("CharsetNaoReconhecido")?;
    let tela = TelaServida {
        url: link.as_str(),
        html: &html,
        codificacao,
    };
    let post = submissao(&instalacao, &tela, &id, pedido.formato, pedido.unidade)?;
    let RespostaDoEnvio::Html(resposta) =
        enviar_submissao_via(post, sessao.cookie, |submissao, cookie| {
            rede.post(submissao, cookie)
        })?
    else {
        bail!("GeracaoRedirecionada");
    };
    let url = endereco_download(&instalacao, &resposta, &id)?;
    let binario = rede.get_binario(&url, &cookie, pedido.limite.para_o_transporte())?;
    let arquivo = gravar(binario, pedido.formato, pedido.saida, pedido.limite)?;
    Ok((arquivo, rede.feitas()))
}

pub(crate) fn tentar_exportar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Exportar {
                numero,
                formato,
                saida,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(crate::processo_concluir_cli::com_sessao(
        cli,
        |sessao, indice| {
            let limite = LimiteDoAnexo::do_ambiente().context("LimiteDeDownloadInvalido")?;
            conferir_diretorios(saida, Politica::exclusiva().diretorio_do_usuario())?;
            let pedido = Pedido {
                numero,
                unidade: cli.unidade.as_deref(),
                formato: *formato,
                saida,
                limite,
            };
            let (arquivo, requisicoes) = exportar(sessao, &pedido, indice, &mut EnvioHttps)?;
            if cli.json {
                let mut valor = serde_json::json!({
                    "numero": numero.as_str(),
                    "formato": formato,
                    "arquivo": saida,
                    "tamanho_bytes": arquivo.tamanho_bytes,
                    "sha256": arquivo.sha256,
                    "requisicoes": requisicoes,
                });
                crate::indice_local::anexar_avisos(&mut valor, indice);
                println!("{valor}");
            } else {
                println!(
                    "Exportado {}: {} bytes.",
                    numero.as_str(),
                    arquivo.tamanho_bytes
                );
            }
            Ok(())
        },
    ))
}

#[cfg(test)]
mod acervo;
#[cfg(test)]
mod tests;
