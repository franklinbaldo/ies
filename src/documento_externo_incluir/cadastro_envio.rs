//! BDD do POST definitivo de documento externo derivado da tela recarregada (#332).
//! Só prepara SubmissaoDerivada; nunca transmite uma escrita.
use super::{anexo::FalhaAnexo, cadastro::CadastroExterno, recibo::ReciboUpload};
use crate::formulario_web::{ErroSubmissaoFormulario, SubmissaoDerivada, TelaServida};
use crate::instalacao::Instalacao;

#[derive(Debug)]
pub(crate) enum FalhaCadastroEnvio {
    ReciboSemData,
    Anexo(FalhaAnexo),
    Formulario(ErroSubmissaoFormulario),
}

pub(crate) fn preparar_cadastro_definitivo(
    _instalacao: &Instalacao,
    _tela: &TelaServida<'_>,
    _unidade: &str,
    _cadastro: &CadastroExterno,
    _recibo: &ReciboUpload,
) -> Result<SubmissaoDerivada, FalhaCadastroEnvio> {
    Err(FalhaCadastroEnvio::ReciboSemData)
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

    fn preparar(html: &str, recibo: &ReciboUpload) -> Result<SubmissaoDerivada, FalhaCadastroEnvio> {
        let instalacao = Instalacao::interpretar(a::ORIGEM).unwrap();
        let cadastro = observar_cadastro(html, "Anexo", 68).unwrap();
        let url = format!("{}/controlador.php?acao=documento_receber&id_procedimento=10&infra_hash={}", a::ORIGEM, "a".repeat(64));
        let tela = TelaServida { url: &url, html, codificacao: CodificacaoDaTela::Iso88591 };
        preparar_cadastro_definitivo(&instalacao, &tela, a::UNIDADE, &cadastro, recibo)
    }

    #[test]
    fn dado_recibo_temporario_quando_preparo_cadastro_entao_flag_2_anexo_e_dia_do_sei() {
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
    fn dado_recibo_sem_data_quando_preparo_cadastro_entao_recuso_antes_do_post() {
        let mut r = recibo();
        r.data_hora = "sem-data".into();
        let pagina = html();
        assert!(matches!(preparar(&pagina, &r), Err(FalhaCadastroEnvio::ReciboSemData)));
    }
}
