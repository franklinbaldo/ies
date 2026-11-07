//! #332 BDD-332-03/04: conecta o formulário Web observado à sequência única.
//! Transporte de upload é injetado; o comando público não autoriza escrita
//! até a validação em treinamento e a confirmação segura do fluxo completo.
use super::{
    PdfLocal,
    confirmacao::{FalhaConfirmacao, FotoProcesso, confirmar_sequencia},
    fluxo::{ErroPreviaExterna, preparar_previa},
    recibo::{FalhaUpload, ReciboUpload},
};
use crate::formulario_web::{CodificacaoDaTela, TelaServida, enviar_submissao_via};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::leitura;
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso, obter_processo};
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};

pub(crate) struct InclusaoComprovada {
    pub(crate) numero_sei: String,
    pub(crate) unidade: String,
}

#[derive(Debug)]
pub(crate) enum ErroConfirmacaoExterna {
    Previa(ErroPreviaExterna),
    EnderecoDeUploadNaoObservado,
    Sequencia(FalhaConfirmacao),
}
impl ErroConfirmacaoExterna {
    pub(crate) fn codigo_saida(&self) -> u8 {
        match self {
            Self::Previa(_) | Self::EnderecoDeUploadNaoObservado => 1,
            Self::Sequencia(e) => e.codigo_saida(),
        }
    }

    pub(crate) fn etapa(&self) -> &'static str {
        match self {
            Self::Previa(_) | Self::EnderecoDeUploadNaoObservado => "previa",
            Self::Sequencia(
                FalhaConfirmacao::FotoInicialIncompleta
                | FalhaConfirmacao::FotoInicialInconsistente,
            ) => "fotografia_inicial",
            Self::Sequencia(
                FalhaConfirmacao::UploadRecusado(_) | FalhaConfirmacao::UploadIncerto(_),
            ) => "upload_temporario",
            Self::Sequencia(FalhaConfirmacao::TemporarioSemCadastro(_)) => "preparo_do_cadastro",
            Self::Sequencia(FalhaConfirmacao::CadastroIncerto(_)) => "cadastro_definitivo",
            Self::Sequencia(
                FalhaConfirmacao::ReleituraIncerta(_) | FalhaConfirmacao::EfeitoIncerto,
            ) => "releitura_independente",
        }
    }

    pub(crate) fn temporario_possivel(&self) -> bool {
        matches!(self, Self::Sequencia(e) if e.temporario_possivel())
    }
}

/// A URL provém exclusivamente do script servido e fica privada.
/// Aceita a ausência observada de `id_procedimento` no endpoint de upload.
fn conferir_upload(
    instalacao: &Instalacao,
    relativo: &str,
) -> Result<String, ErroConfirmacaoExterna> {
    let erro = || ErroConfirmacaoExterna::EnderecoDeUploadNaoObservado;
    let consulta = relativo.strip_prefix("controlador.php?").ok_or_else(erro)?;
    if relativo.len() > 8_192 || !relativo.bytes().all(|b| (33..=126).contains(&b)) {
        return Err(erro());
    }
    let mut acao = 0;
    let mut hash = 0;
    for par in consulta.split('&') {
        let (chave, valor) = par.split_once('=').ok_or_else(erro)?;
        conferir_chave_de_consulta(chave).map_err(|_| erro())?;
        conferir_valor_de_consulta(valor).map_err(|_| erro())?;
        match chave {
            "acao" if valor == "documento_upload_anexo" => acao += 1,
            "infra_hash" if valor.len() == 64 && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hash += 1;
            }
            "acao" | "infra_hash" | "acao_ajax" => return Err(erro()),
            _ => {}
        }
    }
    if acao != 1 || hash != 1 {
        return Err(erro());
    }
    Ok(format!("{}/{relativo}", instalacao.url_base()))
}

/// Navega até o cadastro atual, tenta um upload, depois um cadastro e uma
/// releitura. A closure de upload representa UMA tentativa, nunca um retry.
/// O chamador só deve oferecer a rota de escrita quando validada em TREINAMENTO.
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
) -> Result<InclusaoComprovada, ErroConfirmacaoExterna>
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
    let preparado = preparar_previa(sessao, nup, unidade_pedida, tipo, pdf, indice, envio)
        .map_err(ErroConfirmacaoExterna::Previa)?;
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(ErroConfirmacaoExterna::EnderecoDeUploadNaoObservado)?;
    let destino = conferir_upload(&instalacao, preparado.cadastro.plano.url_upload())?;
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
    )
    .map_err(ErroConfirmacaoExterna::Sequencia)?;
    Ok(InclusaoComprovada {
        numero_sei,
        unidade,
    })
}

/// Ponte real da sequência #332: o mesmo POST multipart único do transporte,
/// seguido de cadastro único e releitura pelo fluxo já ensaiado offline.
/// Não é invocada pela CLI antes da validação em TREINAMENTO.
#[allow(dead_code, clippy::too_many_arguments)]
pub(crate) fn executar_confirmacao_https<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade_pedida: Option<&str>,
    tipo: &str,
    pdf: &PdfLocal,
    arquivo: impl std::io::Read,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<InclusaoComprovada, ErroConfirmacaoExterna> {
    let cookie = sessao
        .cookie
        .valor()
        .map_err(|e| ErroConfirmacaoExterna::Previa(ErroPreviaExterna::Transporte(e)))?;
    // A origem já foi validada quando a sessão foi aberta.
    let origem = sessao
        .origem
        .split('/')
        .take(3)
        .collect::<Vec<_>>()
        .join("/");
    executar_confirmacao_via(
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
