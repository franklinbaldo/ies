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

/// Uma lista da caixa na forma única (shim do contrato; GREEN a seguir).
pub(crate) fn lista_da_categoria(
    _leitura: &Leitura,
    _categoria: crate::dominio::Categoria,
    _origem: &str,
) -> Value {
    Value::Null
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginas::juntar;
    use std::collections::BTreeSet;

    fn chaves(valor: &Value) -> BTreeSet<&str> {
        valor
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect()
    }

    #[test]
    fn json_publica_somente_dados_sem_sinal_de_completude() {
        let html = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let leitura = juntar(&[html.into()], None).unwrap();
        let listas = json(&leitura, "unb");
        for lista in listas.as_array().unwrap() {
            assert!(lista.get("completa").is_none());
            assert!(lista.get("linhas_lidas").is_none());
            assert!(lista.get("paginas_lidas").is_none());
            assert!(lista["faixa_observada"].is_null());
            assert!(lista.get("faixa_observada").is_some());
        }
    }

    #[test]
    fn caixa_sao_duas_listas_independentes_na_ordem_recebidos_gerados() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>(5 registros - 1 a 1):</caption>\
                    <tr id='P1'><td><a class='processoVisualizado'>N-1</a></td></tr></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let leitura = juntar(&[html.into()], None).unwrap();
        let listas = json(&leitura, "web");
        let listas = listas.as_array().expect("array de listas");
        assert_eq!(listas.len(), 2);
        let forma = BTreeSet::from([
            "categoria",
            "faixa_observada",
            "origem",
            "processos",
            "total_declarado",
            "unidade",
        ]);
        for (lista, categoria) in listas.iter().zip(["recebidos", "gerados"]) {
            assert_eq!(chaves(lista), forma);
            assert_eq!(lista["categoria"], categoria);
            assert_eq!(lista["unidade"], "PPI");
            assert_eq!(lista["origem"], "web");
        }
        assert_eq!(listas[0]["total_declarado"], 5);
        assert_eq!(listas[0]["faixa_observada"]["inicio"], 1);
        assert_eq!(listas[0]["processos"][0]["numero"], "N-1");
        assert_eq!(
            listas[1],
            lista_da_categoria(&leitura, crate::dominio::Categoria::Gerados, "web")
        );
        assert_eq!(
            listas[0],
            lista_da_categoria(&leitura, crate::dominio::Categoria::Recebidos, "web")
        );
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
