//! Entrada de `ies documento ciente DOC [--confirmar]`.
//! A prova é a atividade na Lista de Ciências do documento relida.
use crate::ciencia::{CienciaDoDocumento, PedidoDaCienciaDocumento, ciencia_no_documento};
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain};
use serde_json::json;

pub(crate) fn tentar_ciente_documento(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Documento(Documento {
        acao: Some(DocumentoAcao::Ciente { numero, confirmar }),
        processo,
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDaCienciaDocumento {
            numero,
            processo: processo.as_ref(),
            unidade: cli.unidade.as_deref(),
            confirmar: *confirmar,
        };
        let feito = ciencia_no_documento(sessao, &pedido, indice, &mut EnvioHttps)?;
        sucesso(cli.json, &feito, indice);
        Ok(())
    }))
}

fn sucesso(json_mode: bool, feito: &CienciaDoDocumento, indice: &crate::indice_local::IndiceLocal) {
    if json_mode {
        let mut valor = json!({
            "numero": feito.numero,
            "id_documento": feito.id_documento,
            "rotulo": feito.rotulo,
            "processo": {"nup": feito.processo_nup, "id": feito.id_procedimento},
            "unidade": feito.unidade,
            "origem": "web",
            "confirmado": feito.confirmado,
            "verificado": feito.verificado,
            "id_atividade": feito.id_atividade,
            "usuario": feito.usuario,
            "requisicoes": feito.requisicoes,
        });
        crate::indice_local::anexar_avisos(&mut valor, indice);
        println!("{valor}");
    } else if feito.verificado {
        println!(
            "Ciência no documento {} verificada na Lista de Ciências (atividade {}). Requisições: {}.",
            feito.numero,
            feito.id_atividade.as_deref().unwrap_or("não observada"),
            feito.requisicoes
        );
    } else {
        println!("{}", texto_da_previa(feito));
    }
}

/// Formata a prévia humana usando somente dados já observados na abertura.
/// A unidade é opcional: não inventar localização quando ela não foi publicada.
fn texto_da_previa(feito: &CienciaDoDocumento) -> String {
    let unidade = feito
        .unidade
        .as_deref()
        .filter(|nome| !nome.trim().is_empty())
        .map(|nome| format!(", na unidade {nome}"))
        .unwrap_or_default();
    format!(
        "Prévia: registrar ciência do usuário desta sessão no documento {} ({}) do processo {}{}. O SEI permite uma ciência por documento. Para executar: --confirmar. Requisições: {}.",
        feito.numero, feito.rotulo, feito.processo_nup, unidade, feito.requisicoes
    )
}

#[cfg(test)]
mod testes_previa {
    use super::*;

    fn feito(unidade: Option<&str>) -> CienciaDoDocumento {
        CienciaDoDocumento {
            numero: "0011607".into(),
            id_documento: "21".into(),
            rotulo: "Despacho".into(),
            processo_nup: "0000001-22.2026.4.00.0000".into(),
            id_procedimento: "10".into(),
            unidade: unidade.map(str::to_owned),
            confirmado: false,
            verificado: false,
            id_atividade: None,
            usuario: None,
            requisicoes: 3,
        }
    }

    /// RED #271: documento ciente deve mostrar a unidade efetivamente observada,
    /// como a prévia do Todos, e nunca deixar 14 espaços no meio da frase.
    #[test]
    fn previa_humana_publica_unidade_observada_sem_espacos_duplicados() {
        let texto = texto_da_previa(&feito(Some("Unidade3")));
        assert!(texto.contains("no documento 0011607 (Despacho)"));
        assert!(texto.contains("do processo 0000001-22.2026.4.00.0000, na unidade Unidade3."));
        assert!(texto.contains("Para executar: --confirmar."));
        assert!(texto.contains("Requisições: 3."));
        assert!(!texto.contains("  "), "espaçamento da prévia: {texto}");
    }

    #[test]
    fn previa_humana_nao_inventa_unidade_nao_observada() {
        let texto = texto_da_previa(&feito(None));
        assert!(texto.contains("do processo 0000001-22.2026.4.00.0000."));
        assert!(!texto.contains("na unidade"));
    }
}
