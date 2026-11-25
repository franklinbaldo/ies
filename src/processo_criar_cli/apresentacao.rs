//! Apresentação do resultado confirmado ou prévia de criação do processo (#245).
//! Os erros tipados e as recusas permanecem na fronteira da CLI.
use crate::processo_criar_fluxo::Criacao;
use serde_json::json;

fn informar_causa_post(causa: Option<&str>) {
    if let Some(causa) = causa {
        println!("Causa do POST reconciliado por leitura: {causa}");
    }
}

pub(super) fn informar_sucesso(json_mode: bool, feito: Criacao, especificacao: &str) {
    if json_mode {
        println!(
            "{}",
            json!({
                "tipo": feito.tipo,
                "especificacao": especificacao,
                "assunto": feito.assunto,
                "interessados": feito.interessados,
                "nivel": feito.nivel,
                "hipotese_legal": feito.hipotese_legal,
                "unidade": feito.unidade,
                "confirmado": feito.confirmado,
                "nup": feito.nup,
                "causa_post": feito.causa_post,
                "origem": "web",
                "requisicoes": feito.requisicoes,
            })
        );
    } else {
        println!(
            "IES: {} processo «{}» na unidade {} ({} requisições).",
            if feito.confirmado {
                "criado"
            } else {
                "prévia de criação de"
            },
            feito.tipo,
            feito.unidade,
            feito.requisicoes
        );
        println!("Especificação: {especificacao}");
        println!("Assunto sugerido: {}", feito.assunto);
        if !feito.interessados.is_empty() {
            println!("Interessados: {}", feito.interessados.join("; "));
        }
        println!("Acesso: {}", feito.nivel.como_texto());
        if let Some(hipotese) = feito.hipotese_legal {
            println!("Hipótese legal observada (ID): {hipotese}");
        }
        informar_causa_post(feito.causa_post);
        if let Some(nup) = feito.nup {
            println!("NUP criado e verificado: {nup}");
        } else {
            println!("Para criar, repita com --confirmar.");
        }
    }
}
