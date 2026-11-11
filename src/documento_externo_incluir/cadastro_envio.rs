//! Preparação técnica do cadastro externo a partir do formulário recarregado (#332).
//! Só prepara `SubmissaoDerivada`; nunca transmite uma escrita.
use super::{anexo::FalhaAnexo, cadastro::CadastroExterno, recibo::ReciboUpload};
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
    derivar_submissao,
};
use crate::instalacao::Instalacao;

const CADASTRAR_EXTERNO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::DocumentoReceber,
    formulario: "frmDocumentoCadastro",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnIdSerie", "hdnAnexos", "hdnFlagDocumentoCadastro"],
    campos_ocultos_reescritos: &["hdnAnexos", "hdnFlagDocumentoCadastro"],
    campos_usuario: &["txtDataElaboracao", "rdoFormato", "rdoNivelAcesso"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::DocumentoCriadoNoProcesso,
    },
};

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaCadastroEnvio {
    ReciboSemData,
    Anexo(#[source] FalhaAnexo),
    Formulario(#[source] ErroSubmissaoFormulario),
}

pub(crate) fn preparar_cadastro_definitivo(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    cadastro: &CadastroExterno,
    recibo: &ReciboUpload,
) -> Result<SubmissaoDerivada, FalhaCadastroEnvio> {
    // Na UnB, o dia que o cadastro exige vem do recibo do servidor,
    // não do relógio do cliente (que pode estar em outro fuso).
    let (dia, hora) = recibo
        .data_hora
        .split_once(' ')
        .ok_or(FalhaCadastroEnvio::ReciboSemData)?;
    let bytes = dia.as_bytes();
    if bytes.len() != 10
        || hora.is_empty()
        || !bytes.iter().enumerate().all(|(i, b)| {
            if i == 2 || i == 5 {
                *b == b'/'
            } else {
                b.is_ascii_digit()
            }
        })
    {
        return Err(FalhaCadastroEnvio::ReciboSemData);
    }
    let linha = cadastro
        .plano
        .linha_do_anexo(recibo)
        .map_err(FalhaCadastroEnvio::Anexo)?;
    derivar_submissao(
        &CADASTRAR_EXTERNO,
        instalacao,
        tela,
        unidade,
        &[
            ("hdnFlagDocumentoCadastro", "2"),
            ("hdnAnexos", &linha),
            ("txtDataElaboracao", dia),
            ("rdoFormato", "N"),
            ("rdoNivelAcesso", "0"),
        ],
    )
    .map_err(FalhaCadastroEnvio::Formulario)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documento_externo_incluir::cadastro::observar_cadastro;
    use crate::formulario_web::CodificacaoDaTela;
    use crate::processo_concluir::tests_apoio as a;

    fn recibo() -> ReciboUpload {
        ReciboUpload {
            nome_upload: "temp-42".into(),
            nome: "prova.pdf".into(),
            tipo: "application/pdf".into(),
            tamanho_bytes: 68,
            data_hora: "11/10/2026 11:07:50".into(),
        }
    }

    fn html() -> String {
        format!(
            r#"<html><body><form id="frmDocumentoCadastro" method="post"
 action="controlador.php?acao=documento_receber&amp;id_procedimento=10&amp;infra_hash={hash}">
 <select id="selSerie" name="selSerie"><option value="293" selected="selected">Anexo</option></select>
 <input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value="293">
 <input type="hidden" id="hdnFlagDocumentoCadastro" name="hdnFlagDocumentoCadastro" value="1">
 <input type="hidden" id="hdnAnexos" name="hdnAnexos" value="">
 <input type="hidden" id="hdnAssuntos" name="hdnAssuntos" value="2062±Assunto observado">
 <input type="text" id="txtDataElaboracao" name="txtDataElaboracao" value="">
 <input type="radio" name="rdoFormato" value="N" checked="checked">
 <input type="radio" name="rdoFormato" value="D">
 <input type="radio" name="rdoNivelAcesso" value="0" checked="checked">
 </form><form id="frmAnexos"><input type="file" name="filArquivo"></form>
 <script>var arrExt = [];
 arrExt[0] = {{nome : "pdf", tamanho : 200}};
 objTabelaAnexos.adicionar([arr['nome_upload'],arr['nome'],arr['data_hora'],arr['tamanho'],infraFormatarTamanhoBytes(arr['tamanho'])]);
 objUpload = new infraUpload('frmAnexos','controlador.php?acao=documento_upload_anexo&infra_hash=SINTETICO');</script></body></html>"#,
            hash = "a".repeat(64)
        )
    }

    fn preparar(
        html: &str,
        recibo: &ReciboUpload,
    ) -> Result<SubmissaoDerivada, FalhaCadastroEnvio> {
        let instalacao = Instalacao::interpretar(a::ORIGEM).unwrap();
        let cadastro = observar_cadastro(html, "Anexo", 68).unwrap();
        let url = format!(
            "{}/controlador.php?acao=documento_receber&id_procedimento=10&infra_hash={}",
            a::ORIGEM,
            "a".repeat(64)
        );
        let tela = TelaServida {
            url: &url,
            html,
            codificacao: CodificacaoDaTela::Iso88591,
        };
        preparar_cadastro_definitivo(&instalacao, &tela, a::UNIDADE, &cadastro, recibo)
    }

    #[test]
    fn recibo_temporario_gera_flag_2_anexo_e_dia_do_sei() {
        // Caso: Todos tests/test_documento_juntar_010.py, UnB SEI 4.
        let pagina = html();
        let envio = preparar(&pagina, &recibo()).unwrap();
        let corpo = std::str::from_utf8(envio.corpo()).unwrap();
        assert!(envio.url().contains("acao=documento_receber"));
        assert!(corpo.contains("hdnFlagDocumentoCadastro=2"));
        assert!(corpo.contains("hdnAnexos=temp-42%B1prova.pdf"));
        assert!(corpo.contains("txtDataElaboracao=11%2F10%2F2026"));
        assert!(corpo.contains("rdoFormato=N"));
        assert!(corpo.contains("rdoNivelAcesso=0"));
        assert!(corpo.contains("hdnAssuntos=2062%B1Assunto+observado"));
    }

    #[test]
    fn recibo_sem_dia_nao_gera_cadastro() {
        let mut r = recibo();
        r.data_hora = "sem-data".into();
        let pagina = html();
        assert!(matches!(
            preparar(&pagina, &r),
            Err(FalhaCadastroEnvio::ReciboSemData)
        ));
    }
}
