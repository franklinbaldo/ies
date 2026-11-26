use super::{HtmlServido, total_do_resultado};

#[test]
fn resultado_seleciona_total_e_nao_tipo() {
    use crate::formulario_web::tests::{HASH_TELA, instalacao};
    let inst = instalacao();
    let tela = HtmlServido::de_teste(
        "<table><caption>Processos gerados no período:</caption>\
         <tr class='totalEstatisticas'><td>TOTAL:</td><td>99</td></tr></table>\
         <table><caption>Processos com andamento aberto na unidade ao final do período:</caption>\
         <tr><td>Tipo</td><td><a onclick=\"abrirDetalhe('controlador.php?acao=estatisticas_detalhar_unidade&amp;tabela_estatisticas=4&amp;id_tipo_procedimento=1');\">7</a></td></tr>\
         <tr class='totalEstatisticas'><td>TOTAL:</td><td><a onclick=\"abrirDetalhe('controlador.php?acao=estatisticas_detalhar_unidade&amp;tabela_estatisticas=4');\">1.234</a></td></tr></table>",
    );
    let tela = HtmlServido::de_teste(&tela.as_str().replace(
        "controlador.php?",
        &format!("controlador.php?infra_hash={HASH_TELA}&"),
    ));
    let (total, url) = total_do_resultado(&tela, &inst).unwrap();
    assert!(url.starts_with(&format!("{}/controlador.php?", inst.url_base())));
    assert_eq!(total, 1234);
    for alterada in [
        tela.as_str().replace("4');", "1');"),
        tela.as_str()
            .replace("4');", "4&amp;id_tipo_procedimento=1');"),
        tela.as_str().replace("1.234", "ilegível"),
        tela.as_str().replace(HASH_TELA, "x"),
        tela.as_str()
            .replace("controlador.php?", "https://fora.example/controlador.php?"),
    ] {
        assert!(total_do_resultado(&HtmlServido::de_teste(&alterada), &inst).is_err());
    }
}

