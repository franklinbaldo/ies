//! Regressão #496, ligada ao BDD-60-05: lista parcial não é ausência provada.
//! Hipótese sintética de PASTA/AGUARDE; cassettes `UnB` de busca não trazem
//! esta recusa. Duas CLIs reais consultam o mesmo SQLite, por TLS loopback.
use super::*;

/// O processo informa leitura parcial; a consulta documental estrita precisa
/// recusar sem substituir `DOC_B` pelo `DOC_A` visível e sem buscar conteúdo.
/// A opção de ocultar inacessíveis não transforma erro de identidade em omissão.
#[test]
fn regressao_496_documento_em_arvore_parcial_recusa_sem_falso_ausente() {
    for ocultar in [false, true] {
        let mut argumentos = vec![
            "--json",
            "documento",
            DOC_B,
            "--processo",
            NUP_A,
            "--sem-cache",
        ];
        if ocultar {
            argumentos.push("--ocultar-inacessiveis");
        }
        let (saida, trilha) = apoio::executar_pasta_pendente(&argumentos);
        falha_por_cli::falha_por(&saida, "Documento", "Processo");
        let stderr = String::from_utf8_lossy(&saida.stderr);
        assert!(
            stderr.contains("PastasNaoCarregadasSemAcaoDeAbrir"),
            "a recusa deve preservar a causa tipada de lista incompleta: {stderr}"
        );
        assert!(
            trilha
                .iter()
                .filter(|evento| evento.as_str() == "GET procedimento_visualizar")
                .count()
                >= 2,
            "processo informativo e documento estrito devem ler a árvore: {trilha:?}"
        );
        assert!(
            !trilha
                .iter()
                .any(|evento| evento == "GET documento_visualizar"),
            "documento alheio nunca deve ser baixado após a leitura parcial: {trilha:?}"
        );
        assert!(
            trilha
                .iter()
                .all(|evento| evento.starts_with("GET ")
                    || evento == "POST protocolo_pesquisa_rapida"),
            "somente leitura e pesquisa, nenhuma escrita ou retry: {trilha:?}"
        );
    }
}
