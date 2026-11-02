//! Apresentação da leitura: JSON v2 e formato humano são projeções,
//! não campos persistidos em Caixa, Tabela ou Processo.
use crate::dominio::{Processo, Tabela};
use crate::paginas::{Cobertura, Leitura};
use serde_json::{Value, json};

/// Posições lidas (`{inicio, fim}`) ou `null` sem linhas: dado observado
/// da legenda, nunca afirmação de lista integral.
pub(crate) fn faixa_json(faixa: Option<(usize, usize)>) -> Value {
    faixa.map_or(
        Value::Null,
        |(inicio, fim)| json!({"inicio": inicio, "fim": fim}),
    )
}

fn tabela_json(tabela: &Tabela<Processo>, cobertura: &Cobertura) -> Value {
    json!({
        "total_declarado": tabela.total_declarado,
        "faixa_observada": faixa_json(cobertura.faixa_observada()),
        "processos": tabela.processos,
    })
}

pub fn json(leitura: &Leitura, origem: &str) -> Value {
    json!({
        "unidade": leitura.unidade,
        "origem": origem,
        "recebidos": tabela_json(&leitura.caixa.recebidos, &leitura.recebidos),
        "gerados": tabela_json(&leitura.caixa.gerados, &leitura.gerados),
    })
}

fn linha_humana(nome: &str, tabela: &Tabela<Processo>, cobertura: &Cobertura) -> String {
    let total = tabela
        .total_declarado
        .map_or_else(|| "desconhecido".to_owned(), |n| n.to_string());
    let faixa = cobertura
        .faixa_observada()
        .map_or_else(|| "nenhuma".to_owned(), |(i, f)| format!("{i} a {f}"));
    format!(
        "{nome}: {} processo(s) observado(s); total declarado pelo SEI: {total}; faixa observada: {faixa}",
        tabela.processos.len()
    )
}

pub fn humano(leitura: &Leitura) -> String {
    format!(
        "{}\n{}",
        linha_humana("recebidos", &leitura.caixa.recebidos, &leitura.recebidos),
        linha_humana("gerados", &leitura.caixa.gerados, &leitura.gerados)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginas::juntar;

    #[test]
    fn json_publica_somente_dados_sem_sinal_de_completude() {
        let html = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let leitura = juntar(&[html.into()], None).unwrap();
        let obj = json(&leitura, "unb");
        assert!(obj["recebidos"].get("completa").is_none());
        assert!(obj["gerados"].get("completa").is_none());
        assert!(obj.get("completude").is_none());
        assert!(obj["recebidos"].get("linhas_lidas").is_none());
        assert!(obj["recebidos"].get("paginas_lidas").is_none());
        assert!(obj["recebidos"]["faixa_observada"].is_null());
        assert!(obj["recebidos"].get("faixa_observada").is_some());
    }

    #[test]
    fn humano_informa_faixa_observada_e_total() {
        let html = "<table id='tblProcessosRecebidos'><caption>(5 registros - 1 a 1):</caption>\
                    <tr id='P1'><td><a class='processoVisualizado'>N-1</a></td></tr></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let leitura = juntar(&[html.into()], None).unwrap();
        let texto = humano(&leitura);
        assert!(
            texto.contains("total declarado pelo SEI: 5; faixa observada: 1 a 1"),
            "{texto}"
        );
        assert!(texto.contains("gerados: 0 processo(s) observado(s); total declarado pelo SEI: 0; faixa observada: nenhuma"), "{texto}");
    }
}
