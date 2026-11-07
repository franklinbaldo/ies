//! #332 — plano do anexo externo derivado da tela, sem executar JavaScript.
//! O recibo temporário não comprova o cadastro do documento.

use super::recibo::ReciboUpload;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FalhaAnexo {
    FormaNaoObservada,
    LinhaInvalida,
    ReciboIncompativel,
}

pub(crate) struct PlanoAnexo {
    upload: String,
}

impl PlanoAnexo {
    pub(crate) fn url_upload(&self) -> &str {
        &self.upload
    }

    pub(crate) fn linha_do_anexo(
        &self,
        _recibo: &ReciboUpload,
    ) -> Result<String, FalhaAnexo> {
        Err(FalhaAnexo::LinhaInvalida)
    }
}

/// Lê apenas declarações literais da tela de cadastro já servida.
pub(crate) fn ler_plano(_html: &str) -> Result<PlanoAnexo, FalhaAnexo> {
    Err(FalhaAnexo::FormaNaoObservada)
}

#[cfg(test)]
mod testes {
    use super::*;

    const COLUNAS: &str = "arr['nome_upload'],arr['nome'],arr['data_hora'],arr['tamanho'],infraFormatarTamanhoBytes(arr['tamanho']),'operador','Unidade2'";

    fn tela(terceiro: &str, colunas: &str) -> String {
        format!(
            r#"<html><body><form id="frmDocumentoCadastro"><input type="hidden" name="hdnAnexos" /></form>
<script type="text/javascript">
funcaoConclusao = function(arr) {{
  objTabelaAnexos.adicionar([{colunas}]);
}};
objUpload = new infraUpload('frmAnexos','controlador.php?acao=documento_upload_anexo&infra_hash=SINTETICO'{terceiro});
objTabelaAnexos = new infraTabelaDinamica('tblAnexos','hdnAnexos',false,false);
</script></body></html>"#
        )
    }

    fn recibo() -> ReciboUpload {
        ReciboUpload {
            nome_upload: "temp-42".to_owned(),
            nome: "prova.pdf".to_owned(),
            tipo: "application/pdf".to_owned(),
            tamanho_bytes: 68,
            data_hora: "11/10/2026 11:07:50".to_owned(),
        }
    }

    #[test]
    fn dado_cadastro_unb_4_quando_recebo_upload_entao_gera_linha_da_tabela() {
        // Dado a ordem da funcaoConclusao capturada no Todos/UnB,
        // quando monto hdnAnexos, então preservo todas as colunas observadas.
        let plano = ler_plano(&tela("", COLUNAS)).expect("cadastro UnB 4");
        assert!(plano.url_upload().contains("acao=documento_upload_anexo"));
        assert_eq!(
            plano.linha_do_anexo(&recibo()),
            Ok("temp-42±prova.pdf±11/10/2026 11:07:50±68±0.07 Kb±operador±Unidade2".to_owned())
        );
    }

    #[test]
    fn dado_sei_5_quando_terceiro_argumento_booleano_entao_mesmo_plano() {
        // PEN3 SEI 5.0.4 publica new infraUpload(..., true).
        for terceiro in [", true", ", false"] {
            let plano = ler_plano(&tela(terceiro, COLUNAS)).expect("cadastro SEI 5");
            assert_eq!(plano.linha_do_anexo(&recibo()).unwrap().split('±').count(), 7);
        }
    }

    #[test]
    fn dado_colunas_reordenadas_quando_recibo_entao_respeita_a_tela() {
        let colunas = "'Unidade2',arr['nome'],arr['nome_upload'],infraFormatarTamanhoBytes(arr['tamanho']),arr['data_hora']";
        let plano = ler_plano(&tela("", colunas)).expect("ordem da tela");
        assert_eq!(
            plano.linha_do_anexo(&recibo()),
            Ok("Unidade2±prova.pdf±temp-42±0.07 Kb±11/10/2026 11:07:50".to_owned())
        );
    }

    #[test]
    fn dado_upload_ambíguo_ou_nao_observado_quando_leio_entao_recuso() {
        for tela in [
            tela(", objeto", COLUNAS),
            tela(", true, false", COLUNAS),
            tela("", COLUNAS).replace("new infraUpload", "outraCoisa"),
            format!("{}{}", tela("", COLUNAS), tela("", COLUNAS)),
        ] {
            assert!(matches!(ler_plano(&tela), Err(FalhaAnexo::FormaNaoObservada)));
        }
    }

    #[test]
    fn dado_coluna_nao_observada_ou_ambiguo_quando_leio_entao_recuso() {
        for colunas in ["arr['campo_desconhecido']", "", "arr['nome'],funcao()", "arr['nome'"] {
            assert!(matches!(
                ler_plano(&tela("", colunas)),
                Err(FalhaAnexo::LinhaInvalida)
            ));
        }
    }

    #[test]
    fn dado_recibo_com_separador_quando_monto_linha_entao_recuso() {
        let plano = ler_plano(&tela("", COLUNAS)).expect("cadastro");
        let mut resposta = recibo();
        resposta.nome = "prova±outro.pdf".to_owned();
        assert_eq!(
            plano.linha_do_anexo(&resposta),
            Err(FalhaAnexo::ReciboIncompativel)
        );
    }

    #[test]
    fn dado_only_template_quando_leio_entao_nao_invento_upload() {
        let falso = format!("<template>{}</template>", tela("", COLUNAS));
        assert!(matches!(ler_plano(&falso), Err(FalhaAnexo::FormaNaoObservada)));
    }
}
