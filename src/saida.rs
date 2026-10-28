//! Apresentação da leitura: JSON v1 e formato humano são projeções,
//! não campos persistidos em Caixa, Tabela ou Processo.
use crate::dominio::{Processo, Tabela};
use crate::paginas::{Cobertura, Leitura};
use serde_json::{Value, json};

fn tabela_json(tabela: &Tabela<Processo>, cobertura: &Cobertura) -> Value {
    json!({
        "total_declarado": tabela.total_declarado,
        "processos": tabela.processos,
        "completa": cobertura.completa(tabela),
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
    let cobertura = if cobertura.completa(tabela) {
        "cobertura comprovada"
    } else {
        "cobertura não comprovada"
    };
    format!(
        "{nome}: {} de {total} processos ({cobertura})",
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
    fn json_derive_completa_da_cobertura_verificada() {
        let html = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let leitura = juntar(&[html.into()], None).unwrap();
        let obj = json(&leitura, "unb");
        assert_eq!(obj["recebidos"]["completa"], true);
        assert_eq!(obj["gerados"]["completa"], true);
        assert!(obj.get("completude").is_none());
        assert!(obj["recebidos"].get("linhas_lidas").is_none());
        assert!(obj["recebidos"].get("paginas_lidas").is_none());
    }
}
