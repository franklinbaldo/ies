//! Entrada da CLI para `ies processo ciente NUP [--confirmar]`.
//! A prévia não dispara o GET que grava a ciência; o resultado não contém
//! URL, cookie, assinatura ou hash.
use crate::ciencia::{
    CienciaDoProcesso, ErroCienciaProcesso, PedidoDaCiencia, ciencia_no_processo,
};
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::{Value, json};
use std::process::ExitCode;

pub(crate) fn tentar_ciente(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Ciente { numero, confirmar }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(match NumeroProcesso::novo(numero) {
        Ok(nup) => com_sessao(cli, |sessao, indice| {
            let pedido = PedidoDaCiencia {
                nup: &nup,
                unidade: cli.unidade.as_deref(),
                confirmar: *confirmar,
            };
            match ciencia_no_processo(sessao, &pedido, indice, &mut EnvioHttps) {
                Ok(feito) => publicar_sucesso(cli.json, &feito),
                Err(erro) => publicar_erro(cli.json, &erro),
            }
        }),
        Err(_) => erro_de_uso(cli.json),
    })
}

fn json_de_sucesso(feito: &CienciaDoProcesso) -> Value {
    json!({
        "numero": feito.nup,
        "id_procedimento": feito.id_procedimento,
        "unidade": feito.unidade,
        "origem": "web",
        "confirmado": feito.confirmado,
        "verificado": feito.verificado,
        "id_atividade": feito.id_atividade,
        "usuario": feito.usuario,
        "requisicoes": feito.requisicoes,
    })
}

fn publicar_sucesso(json_mode: bool, feito: &CienciaDoProcesso) -> ExitCode {
    if json_mode {
        println!("{}", json_de_sucesso(feito));
    } else if feito.verificado {
        println!(
            "Ciência no processo {} verificada na lista (atividade {}). Requisições: {}.",
            feito.nup,
            feito.id_atividade.as_deref().unwrap_or("não observada"),
            feito.requisicoes
        );
    } else {
        println!(
            "Prévia: registrar ciência no processo {} na unidade {}. Uma ciência por dia, por usuário. Para executar: --confirmar. Requisições: {}.",
            feito.nup, feito.unidade, feito.requisicoes
        );
    }
    ExitCode::SUCCESS
}

fn publicar_erro(json_mode: bool, erro: &ErroCienciaProcesso) -> ExitCode {
    let objeto = json_de_erro(erro);
    if json_mode {
        println!("{objeto}");
    } else {
        eprintln!(
            "{}",
            crate::saida_erro::linha_humana(&erro.mensagem(), erro.status_http())
        );
        if let Some(aviso) = aviso_da_falha(erro) {
            eprintln!("{aviso}");
        }
    }
    ExitCode::from(1)
}

/// Efeito incerto apenas depois do GET de escrita: contagem observada e
/// proibição de repetição; a mensagem tipada indica a prova independente.
fn aviso_da_falha(erro: &ErroCienciaProcesso) -> Option<String> {
    erro.requisicoes()
        .map(|requisicoes| format!("IES: {requisicoes} requisições; não repetir a escrita"))
}

pub(crate) fn json_de_erro(erro: &ErroCienciaProcesso) -> serde_json::Value {
    let codigo = if erro.requisicoes().is_some() {
        "nao_confirmado"
    } else {
        "fonte_invalida"
    };
    let mut objeto = json!({
        "codigo": codigo,
        "erro": erro.mensagem(),
        "retentavel": false,
    });
    if let ErroCienciaProcesso::NaoConfirmado { requisicoes, .. } = erro {
        objeto["requisicoes"] = json!(requisicoes);
    }
    if let Some(http) = erro.status_http() {
        objeto["status_http"] = json!(http.status);
        if let Some(mensagem) = &http.mensagem_do_servidor {
            objeto["mensagem_do_servidor"] = json!(mensagem);
        }
    }
    objeto
}

#[cfg(test)]
mod testes_aviso {
    use super::*;

    /// RED #271: apos um GET potencialmente mutador, a orientacao publica
    /// deve apontar a prova independente, sem sugerir nova escrita.
    #[test]
    fn nao_confirmado_indica_lista_antes_de_nova_confirmacao() {
        let erro = ErroCienciaProcesso::NaoConfirmado {
            motivo: crate::ciencia::VerificacaoDaCiencia::AceiteSemIdDeAtividade,
            requisicoes: 6,
            falha_do_aceite: None,
        };
        let mensagem = erro.mensagem();
        assert!(mensagem.contains("Lista de Ciências"), "{mensagem}");
        assert!(mensagem.contains("antes de nova confirmação"), "{mensagem}");
        assert!(!mensagem.contains("antes de repetir"), "{mensagem}");
        assert_eq!(erro.requisicoes(), Some(6));
        assert_eq!(
            aviso_da_falha(&erro).as_deref(),
            Some("IES: 6 requisições; não repetir a escrita")
        );
        assert_eq!(json_de_erro(&erro)["codigo"], "nao_confirmado");
        assert_eq!(json_de_erro(&erro)["requisicoes"], 6);
    }

    #[test]
    fn falha_antes_da_escrita_nao_exibe_nao_confirmado() {
        let erro = ErroCienciaProcesso::CienciaNaoOferecida;
        assert_eq!(erro.requisicoes(), None);
        assert!(aviso_da_falha(&erro).is_none());
    }
}
