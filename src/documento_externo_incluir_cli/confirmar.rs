//! #332 — apresentação da sequência de inclusão externa na fronteira da CLI.
//! Cenários BDD-332-03/04: sucesso só após releitura; incerteza sai 3.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documento_externo_incluir::{
        confirmacao::FalhaConfirmacao,
        fluxo_confirmar::ErroConfirmacaoExterna,
    };

    #[test]
    fn bdd_332_04_efeito_incerto_informa_etapa_e_residuo_sem_repeticao() {
        let erro = ErroConfirmacaoExterna::Sequencia(FalhaConfirmacao::EfeitoIncerto);
        let resultado = json_da_falha(&erro, true, 7);
        assert_eq!(resultado["confirmado"], false);
        assert_eq!(resultado["etapa"], "releitura_independente");
        assert_eq!(resultado["requisicoes"], 7);
        assert_eq!(resultado["upload_tentado"], true);
        assert_eq!(resultado["cadastro_tentado"], true);
        assert_eq!(resultado["temporario_possivel"], true);
        assert_eq!(resultado["retentavel"], false);
    }
}
