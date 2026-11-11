//! Apresentação humana do resultado do envio.
use super::{ProcessoEnviado, lista_humana};
use crate::processo_enviar::{DesfechoDoEnvio, MetodoRemocao};

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
    if let Some(remocao) = &feito.remocao {
        linhas.push(match remocao.metodo {
            MetodoRemocao::CheckboxNoEnvio => "Remover anotações no envio: sim".to_owned(),
            MetodoRemocao::LimparAnotacao => {
                "Remover anotações: limpar formulário de anotação antes do envio".to_owned()
            }
        });
        let observada = if remocao.antes { "sim" } else { "não" };
        linhas.push(format!("Anotação na origem, antes do envio: {observada}"));
        if let Some(false) = feito.anotacao_depois() {
            linhas.push("Anotação na origem, após o envio: removida (releitura)".to_owned());
        }
    }
    match &feito.desfecho {
        DesfechoDoEnvio::Previa => {
            linhas.push(format!(
                "Depois de enviar: {}",
                refs(&feito.depois_esperado())
            ));
            linhas.push("Para enviar, repita com --confirmar".to_owned());
        }
        DesfechoDoEnvio::Enviado {
            depois,
            reconciliado_por,
        } => {
            linhas.push(format!("Enviado. Aberto em, relido: {}", refs(depois)));
            if let Some(falha) = reconciliado_por {
                linhas.push(format!(
                    "A resposta não foi a esperada ({falha:?}); a leitura confirmou."
                ));
            }
        }
    }
    linhas.push(format!("Requisições: {}", feito.requisicoes));
    linhas.join("\n")
}
