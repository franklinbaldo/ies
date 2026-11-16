//! Apresentação da leitura: JSON v2 e formato humano são projeções,
//! não campos persistidos em Caixa, Tabela ou Processo. Recebidos e
//! Gerados são listas independentes, publicadas na mesma forma.
use crate::dominio::Categoria;
use crate::erro_web::ErroCaixa;
use crate::paginas::{FaixaObservada, Leitura};
use serde::Serialize;
use serde_json::{Value, json};

/// `{"inicio", "fim"}` das posições lidas; `None` para zero linhas, e então
/// a chave `faixa_observada` é omitida (nem `null`, nem `[]`). Dado observado
/// da legenda, nunca afirmação de lista integral.
pub(crate) fn faixa_json(faixa: FaixaObservada) -> Option<Value> {
    match faixa {
        FaixaObservada::SemLinhas => None,
        FaixaObservada::Lidas { inicio, fim } => Some(json!({"inicio": inicio, "fim": fim})),
    }
}

/// Os dados observados de uma lista da caixa. A categoria não está aqui: só
/// as listas de `recebidos`/`gerados` a têm; a visão detalhada não tem.
pub(crate) struct ListaObservada<'a, P: Serialize> {
    pub(crate) unidade: Option<&'a str>,
    pub(crate) origem: &'a str,
    pub(crate) total_declarado: usize,
    pub(crate) faixa_observada: FaixaObservada,
    pub(crate) processos: &'a [P],
}

/// Forma comum de toda lista da caixa (contracts/cli/unidade-recebidos-gerados.md).
pub(crate) fn lista_json<P: Serialize>(lista: &ListaObservada<'_, P>) -> Value {
    let mut objeto = json!({
        "unidade": lista.unidade,
        "origem": lista.origem,
        "total_declarado": lista.total_declarado,
        "processos": lista.processos,
    });
    if let (Some(campos), Some(faixa)) = (objeto.as_object_mut(), faixa_json(lista.faixa_observada))
    {
        campos.insert("faixa_observada".to_owned(), faixa);
    }
    objeto
}

/// A lista de uma categoria da visão resumida, com `categoria` dada pela leitura.
pub(crate) fn lista_da_categoria(
    leitura: &Leitura,
    categoria: Categoria,
    origem: &str,
) -> Result<Value, ErroCaixa> {
    let (tabela, cobertura) = match categoria {
        Categoria::Recebidos => (&leitura.caixa.recebidos, &leitura.recebidos),
        Categoria::Gerados => (&leitura.caixa.gerados, &leitura.gerados),
    };
    let mut lista = lista_json(&ListaObservada {
        unidade: leitura.unidade.as_deref(),
        origem,
        total_declarado: tabela.total_declarado,
        faixa_observada: cobertura.faixa_observada()?,
        processos: &tabela.processos,
    });
    if let Some(objeto) = lista.as_object_mut() {
        objeto.insert("categoria".to_owned(), json!(categoria.rotulo()));
    }
    Ok(lista)
}

/// A caixa resumida: o array `[recebidos, gerados]`, nessa ordem. Só os
/// testes a usam, como oráculo do golden `tests/fixtures/caixa-v2.json`.
#[cfg(test)]
pub fn json(leitura: &Leitura, origem: &str) -> Result<Value, ErroCaixa> {
    Ok(Value::Array(vec![
        lista_da_categoria(leitura, Categoria::Recebidos, origem)?,
        lista_da_categoria(leitura, Categoria::Gerados, origem)?,
    ]))
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
        let html = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
        );
        let leitura = juntar(&[html], None).unwrap();
        let listas = json(&leitura, "unb").unwrap();
        for lista in listas.as_array().unwrap() {
            assert!(lista.get("completa").is_none());
            assert!(lista.get("linhas_lidas").is_none());
            assert!(lista.get("paginas_lidas").is_none());
            // Zero linhas: a chave some, sem `null` nem `[]` no lugar.
            assert!(lista.get("faixa_observada").is_none());
        }
    }

    #[test]
    fn caixa_sao_duas_listas_independentes_na_ordem_recebidos_gerados() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>Processos (5 registros - 1 a 1):</caption>\
                    <tr id='P1'><td><a class='processoVisualizado'>N-1</a></td></tr></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
        );
        let leitura = juntar(&[html], None).unwrap();
        let listas = json(&leitura, "web").unwrap();
        let listas = listas.as_array().expect("array de listas");
        assert_eq!(listas.len(), 2);
        // Recebidos tem linha lida; gerados tem zero linhas e não tem a faixa.
        let com_faixa = BTreeSet::from([
            "categoria",
            "faixa_observada",
            "origem",
            "processos",
            "total_declarado",
            "unidade",
        ]);
        let sem_faixa = BTreeSet::from([
            "categoria",
            "origem",
            "processos",
            "total_declarado",
            "unidade",
        ]);
        let esperadas = [("recebidos", &com_faixa), ("gerados", &sem_faixa)];
        for (lista, (categoria, forma)) in listas.iter().zip(esperadas) {
            assert_eq!(chaves(lista), *forma);
            assert_eq!(lista["categoria"], categoria);
            assert_eq!(lista["unidade"], "PPI");
            assert_eq!(lista["origem"], "web");
        }
        assert_eq!(listas[0]["total_declarado"], 5);
        assert_eq!(listas[0]["faixa_observada"], json!({"inicio": 1, "fim": 1}));
        assert!(listas[1].get("faixa_observada").is_none());
        assert_eq!(listas[0]["processos"][0]["numero"], "N-1");
        assert_eq!(
            listas[1],
            lista_da_categoria(&leitura, crate::dominio::Categoria::Gerados, "web").unwrap()
        );
        assert_eq!(
            listas[0],
            lista_da_categoria(&leitura, crate::dominio::Categoria::Recebidos, "web").unwrap()
        );
    }
}

#[cfg(test)]
#[path = "saida/tests_golden.rs"]
mod tests_golden;
