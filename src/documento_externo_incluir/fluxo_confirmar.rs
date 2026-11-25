//! #332 BDD-332-03/04: conecta o formulário Web observado à sequência única.
//! Transporte de upload é injetado; o comando público não autoriza escrita
//! até a validação em treinamento e a confirmação segura do fluxo completo.
use super::{
    PdfLocal,
    confirmacao::{FotoProcesso, confirmar_sequencia},
    fluxo::{ErroIncluirExterno, preparar_previa},
    recibo::{FalhaUpload, ReciboUpload},
};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{CodificacaoDaTela, TelaServida, enviar_submissao_via};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::leitura;
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso, obter_processo};

pub(crate) struct InclusaoComprovada {
    pub(crate) numero_sei: String,
    pub(crate) unidade: String,
}

/// Campo que a submissão definitiva exige e o cadastro precisa oferecer
/// antes de qualquer upload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CampoExigidoDoCadastro {
    DataElaboracao,
    Formato,
    NivelAcesso,
}

impl CampoExigidoDoCadastro {
    const TODOS: [Self; 3] = [Self::DataElaboracao, Self::Formato, Self::NivelAcesso];

    /// O `name` do controle no `frmDocumentoCadastro`.
    fn nome_do_controle(self) -> &'static str {
        match self {
            Self::DataElaboracao => "txtDataElaboracao",
            Self::Formato => "rdoFormato",
            Self::NivelAcesso => "rdoNivelAcesso",
        }
    }
}

/// A URL provém exclusivamente do script servido e fica privada.
/// Aceita a ausência observada de `id_procedimento` no endpoint de upload.
pub(crate) fn conferir_upload(
    instalacao: &Instalacao,
    relativo: &str,
    processo: &str,
) -> Result<String, ErroIncluirExterno> {
    let consulta = relativo
        .strip_prefix("controlador.php?")
        .ok_or(ErroIncluirExterno::EnderecoDeUploadRecusado)?;
    if relativo.len() > 8_192 || !relativo.bytes().all(|b| (33..=126).contains(&b)) {
        return Err(ErroIncluirExterno::EnderecoDeUploadRecusado);
    }
    let consulta = ConsultaDoControlador::ler(consulta)
        .map_err(ErroIncluirExterno::ConsultaDoUploadForaDaForma)?;
    match consulta
        .valores("id_procedimento")
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => {}
        [informado] if *informado == processo => {}
        _ => return Err(ErroIncluirExterno::UploadParaOutroProcesso),
    }
    if consulta.acao() != Ok(AcaoDoSei::DocumentoUploadAnexo.como_na_url())
        || consulta.valores("acao_ajax").next().is_some()
        || !consulta
            .unico("infra_hash")
            .is_ok_and(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(ErroIncluirExterno::EnderecoDeUploadRecusado);
    }
    Ok(format!("{}/{relativo}", instalacao.url_base()))
}

/// Navega até o cadastro atual, tenta um upload, depois um cadastro e uma
/// releitura. A closure de upload representa UMA tentativa, nunca um retry.
/// O chamador só deve oferecer a rota de escrita quando validada em TREINAMENTO.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn executar_confirmacao_via<V, U>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade_pedida: Option<&str>,
    tipo: &str,
    pdf: &PdfLocal,
    indice: &IndiceLocal,
    envio: &mut V,
    upload: U,
) -> Result<InclusaoComprovada, ErroIncluirExterno>
where
    V: EnvioDoProcesso,
    U: FnOnce(
        &mut V,
        &str,
        &str,
        CodificacaoDaTela,
        &PdfLocal,
    ) -> Result<ReciboUpload, FalhaUpload>,
{
    executar_confirmacao_via_impl(
        sessao,
        nup,
        unidade_pedida,
        tipo,
        pdf,
        indice,
        envio,
        upload,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn executar_confirmacao_via_prevalidada<V, U>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade_pedida: Option<&str>,
    tipo: &str,
    pdf: &PdfLocal,
    indice: &IndiceLocal,
    envio: &mut V,
    upload: U,
) -> Result<InclusaoComprovada, ErroIncluirExterno>
where
    V: EnvioDoProcesso,
    U: FnOnce(
        &mut V,
        &str,
        &str,
        CodificacaoDaTela,
        &PdfLocal,
    ) -> Result<ReciboUpload, FalhaUpload>,
{
    executar_confirmacao_via_impl(
        sessao,
        nup,
        unidade_pedida,
        tipo,
        pdf,
        indice,
        envio,
        upload,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn executar_confirmacao_via_impl<V, U>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade_pedida: Option<&str>,
    tipo: &str,
    pdf: &PdfLocal,
    indice: &IndiceLocal,
    envio: &mut V,
    upload: U,
    prevalidar: bool,
) -> Result<InclusaoComprovada, ErroIncluirExterno>
where
    V: EnvioDoProcesso,
    U: FnOnce(
        &mut V,
        &str,
        &str,
        CodificacaoDaTela,
        &PdfLocal,
    ) -> Result<ReciboUpload, FalhaUpload>,
{
    let preparado = preparar_previa(sessao, nup, unidade_pedida, tipo, pdf, indice, envio)?;
    // O recibo do upload nao deve ser criado se o cadastro ja mostrou
    // faltar um dos campos que a submissao definitiva exigira.
    if prevalidar {
        for campo in CampoExigidoDoCadastro::TODOS {
            if !preparado
                .previa
                .campos_oferecidos
                .iter()
                .any(|nome| nome == campo.nome_do_controle())
            {
                return Err(ErroIncluirExterno::CampoDoCadastroNaoOferecido(campo));
            }
        }
    }
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(ErroIncluirExterno::OrigemDaSessaoNaoInterpretada)?;
    let destino = conferir_upload(
        &instalacao,
        preparado.cadastro.plano.url_upload(),
        &preparado.foto_antes.id,
    )?;
    let tela = TelaServida {
        url: &preparado.url_cadastro,
        html: &preparado.html_cadastro,
        codificacao: preparado.codificacao,
    };
    let unidade = preparado.previa.unidade;
    let numero_sei = confirmar_sequencia(
        envio,
        &instalacao,
        &tela,
        &unidade,
        &preparado.cadastro,
        &preparado.foto_antes,
        tipo,
        |rede| {
            upload(
                rede,
                &destino,
                &preparado.url_cadastro,
                preparado.codificacao,
                pdf,
            )
        },
        |rede, submissao| enviar_submissao_via(submissao, sessao.cookie, |s, c| rede.post(s, c)),
        |rede| {
            obter_processo(sessao, &leitura(nup, unidade_pedida, false), indice, rede)
                .map(|depois| FotoProcesso::da_arvore(&depois.arvore))
        },
    )?;
    Ok(InclusaoComprovada {
        numero_sei,
        unidade,
    })
}

/// Ponte real da sequência #332: o mesmo POST multipart único do transporte,
/// seguido de cadastro único e releitura pelo fluxo já ensaiado offline.
/// A publicação na main exige comprovação em TREINAMENTO antes de liberar a CLI.
#[allow(clippy::too_many_arguments)]
pub(crate) fn executar_confirmacao_https<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade_pedida: Option<&str>,
    tipo: &str,
    pdf: &PdfLocal,
    caminho_pdf: &std::path::Path,
    indice: &IndiceLocal,
    envio: &mut V,
    upload_tentado: &mut bool,
) -> Result<InclusaoComprovada, ErroIncluirExterno> {
    // Validar antes de abrir sessão/rede. É o mesmo descritor que será
    // transmitido, não uma segunda abertura suscetível a substituição.
    let arquivo = super::abrir_pdf_para_envio(caminho_pdf, pdf)?;
    let cookie = sessao.cookie.valor()?;
    // A origem já foi validada quando a sessão foi aberta.
    let origem = sessao
        .origem
        .split('/')
        .take(3)
        .collect::<Vec<_>>()
        .join("/");
    executar_confirmacao_via_prevalidada(
        sessao,
        nup,
        unidade_pedida,
        tipo,
        pdf,
        indice,
        envio,
        move |_, destino, referer, codificacao, pdf| {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos());
            let fronteira = format!("----ies{}-{nanos:x}", std::process::id());
            let plano = super::multipart::preparar_pdf(
                &pdf.nome,
                pdf.tamanho_bytes,
                &fronteira,
                codificacao,
            )
            .map_err(FalhaUpload::PreparacaoMultipart)?;
            *upload_tentado = true;
            super::upload::enviar_https(
                &plano,
                arquivo,
                destino,
                cookie,
                &origem,
                referer,
                &pdf.nome,
                pdf.tamanho_bytes,
            )
        },
    )
}
