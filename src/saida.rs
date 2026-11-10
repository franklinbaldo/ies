//! Apresentação da leitura: JSON v2 e formato humano são projeções,
//! não campos persistidos em Caixa, Tabela ou Processo. Recebidos e
//! Gerados são listas independentes, publicadas na mesma forma.
use crate::dominio::Categoria;
use crate::paginas::Leitura;
use serde::Serialize;
use serde_json::{Value, json};

/// Posições lidas (`{inicio, fim}`) ou `null` sem linhas: dado observado
/// da legenda, nunca afirmação de lista integral.
pub(crate) fn faixa_json(faixa: Option<(usize, usize)>) -> Value {
    faixa.map_or(
        Value::Null,
        |(inicio, fim)| json!({"inicio": inicio, "fim": fim}),
    )
}

/// Os dados observados de uma lista da caixa.
pub(crate) struct ListaObservada<'a, P: Serialize> {
    pub(crate) unidade: Option<&'a str>,
    pub(crate) origem: &'a str,
    /// `None` (JSON `null`) na lista única da visão detalhada.
    pub(crate) categoria: Option<&'a str>,
    pub(crate) total_declarado: Option<usize>,
    pub(crate) faixa_observada: Option<(usize, usize)>,
    pub(crate) processos: &'a [P],
}

/// Forma única de toda lista da caixa
/// (contracts/cli/unidade-recebidos-gerados.md).
pub(crate) fn lista_json<P: Serialize>(lista: &ListaObservada<'_, P>) -> Value {
    json!({
        "unidade": lista.unidade,
        "origem": lista.origem,
        "categoria": lista.categoria,
        "total_declarado": lista.total_declarado,
        "faixa_observada": faixa_json(lista.faixa_observada),
        "processos": lista.processos,
    })
}

/// A lista de uma categoria da visão resumida.
pub(crate) fn lista_da_categoria(leitura: &Leitura, categoria: Categoria, origem: &str) -> Value {
    let (tabela, cobertura) = match categoria {
        Categoria::Recebidos => (&leitura.caixa.recebidos, &leitura.recebidos),
        Categoria::Gerados => (&leitura.caixa.gerados, &leitura.gerados),
    };
    lista_json(&ListaObservada {
        unidade: leitura.unidade.as_deref(),
        origem,
        categoria: Some(categoria.rotulo()),
        total_declarado: tabela.total_declarado,
        faixa_observada: cobertura.faixa_observada(),
        processos: &tabela.processos,
    })
}

/// A caixa resumida: o array `[recebidos, gerados]`, nessa ordem. Só os
/// testes a usam, como oráculo do golden `tests/fixtures/caixa-v2.json`.
#[cfg(test)]
pub fn json(leitura: &Leitura, origem: &str) -> Value {
    Value::Array(vec![
        lista_da_categoria(leitura, Categoria::Recebidos, origem),
        lista_da_categoria(leitura, Categoria::Gerados, origem),
    ])
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
}

#[cfg(test)]
#[path = "saida/tests_golden.rs"]
mod tests_golden;
