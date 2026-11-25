//! Projeção das leituras opcionais do `processo obter`: propriedades,
//! marcadores e andamentos, em JSON e em texto.
use crate::historico_processo::AndamentosLidos;
use crate::marcadores_processo::{MarcadoresLidos, rotulo_do_marcador};
use crate::processo_obter::{Marcadores, Propriedades};
use crate::propriedades_processo::PropriedadesLidas;
use serde_json::{Value, json};
use std::fmt::Write as _;

/// As propriedades lidas; ids de assunto e de contato não são publicados.
pub(super) fn json_das_propriedades(lidas: &PropriedadesLidas) -> Value {
    let assuntos: Vec<Value> = lidas
        .assuntos
        .iter()
        .map(|a| json!({"codigo": a.codigo, "descricao": a.descricao}))
        .collect();
    let mut objeto = serde_json::Map::new();
    objeto.insert("tipo".to_owned(), json!(lidas.tipo));
    objeto.insert("especificacao".to_owned(), json!(lidas.especificacao));
    objeto.insert("interessados".to_owned(), json!(lidas.interessados));
    objeto.insert("assuntos".to_owned(), Value::Array(assuntos));
    objeto.insert("observacoes".to_owned(), json!(lidas.observacoes));
    objeto.insert(
        "nivel_de_acesso".to_owned(),
        json!(lidas.nivel_de_acesso.como_texto()),
    );
    // Sem hipótese legal, a chave sai (não há `null`).
    if let Some(hipotese) = &lidas.hipotese_legal {
        objeto.insert("hipotese_legal".to_owned(), json!(hipotese));
    }
    objeto.insert("data_de_geracao".to_owned(), json!(lidas.data_de_geracao));
    objeto.insert("fonte".to_owned(), json!(lidas.fonte.acao().como_na_url()));
    Value::Object(objeto)
}

/// Os marcadores lidos (sempre lista), o total da legenda e a faixa lida.
pub(super) fn inserir_marcadores(
    objeto: &mut serde_json::Map<String, Value>,
    lidos: &MarcadoresLidos,
) {
    let itens: Vec<Value> = lidos
        .itens
        .iter()
        .map(|m| {
            let mut marcador = serde_json::Map::new();
            marcador.insert("id".to_owned(), json!(m.id));
            marcador.insert(
                "rotulo".to_owned(),
                json!(rotulo_do_marcador(&m.id, &m.nome)),
            );
            marcador.insert("nome".to_owned(), json!(m.nome));
            marcador.insert("cor".to_owned(), json!(m.cor));
            if let Some(texto) = &m.texto {
                marcador.insert("texto".to_owned(), json!(texto));
            }
            if let Some(usuario) = &m.usuario {
                marcador.insert("usuario".to_owned(), json!(usuario));
            }
            marcador.insert("data_hora".to_owned(), json!(m.data_hora));
            Value::Object(marcador)
        })
        .collect();
    // Zero marcadores: a faixa lida não existe e a chave é omitida.
    let faixa = (!itens.is_empty()).then(|| json!({"inicio": 1, "fim": itens.len()}));
    objeto.insert("marcadores".to_owned(), Value::Array(itens));
    objeto.insert(
        "marcadores_total_declarado".to_owned(),
        json!(lidos.total_declarado),
    );
    if let Some(faixa) = faixa {
        objeto.insert("marcadores_faixa_observada".to_owned(), faixa);
    }
}

/// Linhas do texto humano dos marcadores: uma por marcador.
pub(super) fn humano_dos_marcadores(marcadores: &Marcadores, texto: &mut String) {
    let Marcadores::Lidos(lidos) = marcadores else {
        if let Some(motivo) = marcadores.motivo() {
            let _ = write!(texto, "\nMarcadores não lidos ({motivo}).");
        }
        return;
    };
    let _ = write!(
        texto,
        "\nMarcadores: {} de {} declarados{}",
        lidos.itens.len(),
        lidos.total_declarado,
        if lidos.itens.is_empty() { "." } else { ":" }
    );
    for m in &lidos.itens {
        let _ = write!(
            texto,
            "\n- {} [{}]",
            rotulo_do_marcador(&m.id, &m.nome),
            m.cor
        );
        if let Some(t) = &m.texto {
            let _ = write!(texto, ": {t}");
        }
    }
}

/// Linhas do texto humano das propriedades.
pub(super) fn humano_das_propriedades(propriedades: &Propriedades, texto: &mut String) {
    let Propriedades::Lidas(lidas) = propriedades else {
        if let Some(motivo) = propriedades.motivo() {
            let _ = write!(texto, "\nPropriedades não lidas ({motivo}).");
        }
        return;
    };
    let _ = write!(
        texto,
        "\nPropriedades ({}): tipo {}, nível {}, gerado em {}.",
        lidas.fonte.acao().como_na_url(),
        lidas.tipo,
        lidas.nivel_de_acesso.como_texto(),
        lidas.data_de_geracao,
    );
    if let Some(hipotese) = &lidas.hipotese_legal {
        let _ = write!(texto, "\nHipótese legal: {hipotese}");
    }
    let _ = write!(texto, "\nEspecificação: {}", lidas.especificacao);
    for a in &lidas.assuntos {
        let _ = write!(texto, "\n- assunto {} - {}", a.codigo, a.descricao);
    }
    for interessado in &lidas.interessados {
        let _ = write!(texto, "\n- interessado {interessado}");
    }
    if !lidas.observacoes.is_empty() {
        let _ = write!(texto, "\nObservações: {}", lidas.observacoes);
    }
}

pub(super) fn json_dos_andamentos(lidos: &AndamentosLidos) -> Value {
    // Zero andamentos: a faixa lida não existe e a chave é omitida.
    let faixa = (!lidos.itens.is_empty()).then(|| json!({"inicio": 1, "fim": lidos.itens.len()}));
    let itens: Vec<Value> = lidos
        .itens
        .iter()
        .map(|a| {
            let mut andamento = serde_json::Map::new();
            andamento.insert("ocorrido_em".to_owned(), json!(a.ocorrido_em));
            if let Some(unidade) = &a.unidade {
                andamento.insert("unidade".to_owned(), json!(unidade));
            }
            if let Some(usuario) = &a.usuario {
                andamento.insert("usuario".to_owned(), json!(usuario));
            }
            andamento.insert("descricao".to_owned(), json!(a.descricao));
            Value::Object(andamento)
        })
        .collect();
    let mut objeto = json!({
        "tipo": lidos.tipo.como_texto(),
        "total_declarado": lidos.total_declarado,
        "itens": itens,
    });
    if let (Some(campos), Some(faixa)) = (objeto.as_object_mut(), faixa) {
        campos.insert("faixa_observada".to_owned(), faixa);
    }
    objeto
}
