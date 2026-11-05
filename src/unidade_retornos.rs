//! Leitura dos retornos programados da unidade: duas tabelas do mes, sem escrita.
//! Caso de teste: Todos tests/test_unidade_listar_retornos_010.py @323eb01.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Retorno {
    pub(crate) situacao: &'static str,
    pub(crate) numero: String,
    pub(crate) unidade: String,
    pub(crate) data_programada: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroRetornos {
    TelaDeRetornosDesconhecida,
    ListaDeRetornosIncompleta,
}
pub(crate) fn extrair_retornos(_html: &str) -> Result<Vec<Retorno>, ErroRetornos> {
    Err(ErroRetornos::TelaDeRetornosDesconhecida) // RED: leitor ainda ausente
}
#[cfg(test)]
mod tests {
    use super::*;
    fn tabela(situacao: &str, numero: &str, unidade: &str) -> String {
        format!(
            "<table summary='{situacao}'><caption>1 registro:</caption><thead><tr><th>Processo</th><th>Unidade</th><th>Data Programada</th></tr></thead><tbody><tr><td><a>{numero}</a></td><td>{unidade}</td><td>15/10/2026</td></tr></tbody></table>"
        )
    }
    #[test]
    fn mes_com_duas_situacoes_preserva_linhas_e_ordem() {
        // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
        let html = format!("{}{}", tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1"), tabela("Para Devolver", "NUP-B", "U2"));
        let itens = extrair_retornos(&html).unwrap();
        assert_eq!(itens.len(), 2);
        assert_eq!(itens[0].situacao, "aguardando");
        assert_eq!(itens[0].numero, "NUP-A");
        assert_eq!(itens[1].situacao, "devolver");
        assert_eq!(itens[1].unidade, "U2");
    }
    #[test]
    fn mes_vazio_exige_aviso_estrutural() {
        // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
        let html = "<div id='divAguardandoAreaTabela'>Nenhum registro encontrado.</div>";
        assert_eq!(extrair_retornos(html).unwrap(), Vec::<Retorno>::new());
        assert_eq!(extrair_retornos("<div>outra tela</div>"), Err(ErroRetornos::TelaDeRetornosDesconhecida));
    }
    #[test]
    fn legenda_maior_que_as_linhas_nao_publica_resultado_parcial() {
        let html = tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1").replace("1 registro:", "2 registros:");
        assert_eq!(extrair_retornos(&html), Err(ErroRetornos::ListaDeRetornosIncompleta));
    }
    #[test]
    fn tabelas_duplicadas_sao_recusadas() {
        let t = tabela("Para Devolver", "NUP-A", "U1");
        assert_eq!(extrair_retornos(&format!("{t}{t}")), Err(ErroRetornos::TelaDeRetornosDesconhecida));
    }
}
