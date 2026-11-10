//! Entrada de `ies documento ciente DOC [--confirmar]`.
//! A prova é a atividade na Lista de Ciências do documento relida.
use crate::ciencia::{
    CienciaDoDocumento, ErroCienciaDocumento, PedidoDaCienciaDocumento, ciencia_no_documento,
};
use crate::documento::NumeroDocumento;
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_ciente_documento(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(Documento {
        acao: Some(DocumentoAcao::Ciente { numero, confirmar }),
        processo,
        ..
    }) = &cli.command
    else {
        return None;
    };
    let Ok(numero) = NumeroDocumento::novo(numero) else {
        return Some(erro_de_uso(cli.json));
    };
    let processo = match processo.as_deref().map(NumeroProcesso::novo) {
        None => None,
        Some(Ok(p)) => Some(p),
        Some(Err(_)) => return Some(erro_de_uso(cli.json)),
    };
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDaCienciaDocumento {
            numero: &numero,
            processo: processo.as_ref(),
            unidade: cli.unidade.as_deref(),
            confirmar: *confirmar,
        };
        match ciencia_no_documento(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(feito) => sucesso(cli.json, &feito),
            Err(erro) => falha(cli.json, &erro),
        }
    }))
}

fn sucesso(json_mode: bool, feito: &CienciaDoDocumento) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
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
            })
        );
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
    ExitCode::SUCCESS
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

fn falha(json_mode: bool, erro: &ErroCienciaDocumento) -> ExitCode {
    let saida = json_da_falha(erro);
    if json_mode {
        println!("{saida}");
    } else {
        eprintln!(
            "{}",
            crate::saida_erro::linha_humana(&erro.mensagem(), erro.status_http())
        );
        if let Some(aviso) = aviso_da_falha(erro) {
            eprintln!("{aviso}");
        }
    }
    ExitCode::from(erro.saida())
}

/// Orienta sobre a escrita incerta com a contagem realmente observada.
/// A prévia é somente leitura: o que não pode ser repetido é a escrita.
fn aviso_da_falha(erro: &ErroCienciaDocumento) -> Option<String> {
    match erro {
        ErroCienciaDocumento::EfeitoIncerto { requisicoes, .. } => Some(format!(
            "IES: {requisicoes} requisições; não repetir a escrita; conferir a Lista de Ciências no SEI antes de qualquer nova confirmação"
        )),
        _ => None,
    }
}

pub(crate) fn json_da_falha(erro: &ErroCienciaDocumento) -> serde_json::Value {
    let mut saida = json!({
        "codigo": if erro.saida() == 3 { "efeito_incerto" } else { "fonte_invalida" },
        "erro": erro.mensagem(),
        "retentavel": false,
    });
    if let ErroCienciaDocumento::EfeitoIncerto { requisicoes, .. } = erro {
        saida["requisicoes"] = json!(requisicoes);
    }
    if let Some(http) = erro.status_http() {
        saida["status_http"] = json!(http.status);
        if let Some(mensagem) = &http.mensagem_do_servidor {
            saida["mensagem_do_servidor"] = json!(mensagem);
        }
    }
    saida
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

    /// RED #271: stderr deve informar o custo real da tentativa de escrita
    /// incerta e não sugerir que a próxima prévia (somente leitura) é proibida.
    #[test]
    fn erro_humano_da_ciencia_documental_informa_requests_e_nao_proibe_previa() {
        let erro = ErroCienciaDocumento::EfeitoIncerto {
            motivo: crate::ciencia::VerificacaoDaCienciaDoDocumento::AceiteSemAtividadeDoDocumento,
            requisicoes: 6,
            falha_do_aceite: None,
        };
        let aviso = aviso_da_falha(&erro).expect("efeito incerto deve ter aviso");
        assert!(aviso.contains("6 requisições"), "aviso observado: {aviso}");
        assert!(
            aviso.contains("não repetir a escrita"),
            "orientação de segurança ausente: {aviso}"
        );
        assert!(
            !aviso.contains("nova prévia"),
            "a prévia de leitura é permitida: {aviso}"
        );
    }

    /// RED #271: o JSON não pode instruir "repetir" a escrita incerta.
    /// O aviso humano já distingue releitura da Lista e nova confirmação.
    #[test]
    fn erro_json_documental_orienta_conferir_lista_antes_de_confirmar() {
        let erro = ErroCienciaDocumento::EfeitoIncerto {
            motivo: crate::ciencia::VerificacaoDaCienciaDoDocumento::AceiteSemAtividadeDoDocumento,
            requisicoes: 6,
            falha_do_aceite: None,
        };
        let publicado = json_da_falha(&erro);
        let mensagem = publicado["erro"].as_str().expect("mensagem JSON");
        assert!(mensagem.contains("Lista de Ciências"), "{mensagem}");
        assert!(mensagem.contains("antes de nova confirmação"), "{mensagem}");
        assert!(!mensagem.contains("antes de repetir"), "{mensagem}");
        assert_eq!(publicado["codigo"], "efeito_incerto");
        assert_eq!(publicado["requisicoes"], 6);
        assert_eq!(erro.saida(), 3);
    }

    #[test]
    fn falha_previa_documental_nao_recebe_aviso_de_escrita() {
        assert!(aviso_da_falha(&ErroCienciaDocumento::CienciaNaoOferecida).is_none());
    }
}
