//! Apresentação humana do resultado do envio.
use super::{ProcessoEnviado, lista_humana};

/// Saída humana do sucesso.
pub(crate) fn humano(feito: &ProcessoEnviado) -> String {
    let refs = |v: &[String]| lista_humana(&v.iter().map(String::as_str).collect::<Vec<_>>());
    let mut linhas = vec![
        format!("Processo {} na unidade {}", feito.nup, feito.unidade),
        format!("Aberto em: {}", refs(&feito.aberto_em)),
        format!("Destino: {}", feito.destino.descricao),
        format!(
            "Manter aberto na unidade atual: {}",
            if feito.manter_aberto { "sim" } else { "não" }
        ),
    ];
    if feito.remover_anotacoes {
        linhas.push(match feito.metodo_remocao {
            Some(crate::processo_enviar::MetodoRemocao::CheckboxNoEnvio) => {
                "Remover anotações no envio: sim".to_owned()
            }
            Some(crate::processo_enviar::MetodoRemocao::LimparAnotacao) => {
                "Remover anotações: limpar formulário de anotação antes do envio".to_owned()
            }
            None => "Remover anotações no envio: sim".to_owned(),
        });
        let observada = match feito.anotacao_antes {
            Some(true) => "sim",
            Some(false) => "não",
            None => "não observada",
        };
        linhas.push(format!("Anotação na origem, antes do envio: {observada}"));
        if let Some(false) = feito.anotacao_depois {
            linhas.push("Anotação na origem, após o envio: removida (releitura)".to_owned());
        }
    }
    match (feito.confirmado, &feito.depois) {
        (false, _) => {
            linhas.push(format!(
                "Depois de enviar: {}",
                refs(&feito.depois_esperado())
            ));
            linhas.push("Para enviar, repita com --confirmar".to_owned());
        }
        (true, depois) => {
            let depois = depois.as_deref().unwrap_or_default();
            linhas.push(format!("Enviado. Aberto em, relido: {}", refs(depois)));
            if let Some(falha) = &feito.reconciliado_por {
                linhas.push(format!(
                    "A resposta não foi a esperada ({falha:?}); a leitura confirmou."
                ));
            }
        }
    }
    linhas.push(format!("Requisições: {}", feito.requisicoes));
    linhas.join("\n")
}
