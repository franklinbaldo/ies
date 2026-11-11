//! Entrada pública de documento criar (#246).
//! A confirmação deriva um único POST do cadastro e só reconhece o efeito
//! depois de releitura da árvore. Não há retry.
use crate::documento_criar::{
    EstadoCriacao, PedidoCriacao, ResultadoCriacao, executar_criacao, validar_nivel_de_confirmacao,
};
use crate::processo_concluir::escrita::Contador;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain};
use serde_json::json;

pub(crate) fn tentar_criar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::Criar {
                processo,
                tipo,
                descricao,
                nivel,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    // O nível de escrita é pré-condição independente da sessão: a recusa
    // vem antes de abrir arquivos privados.
    if *confirmar && let Err(erro) = validar_nivel_de_confirmacao(nivel.as_deref()) {
        return Some(Err(erro.into()));
    }
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoCriacao {
            nup: processo,
            unidade: cli.unidade.as_deref(),
            tipo,
            descricao: descricao.as_deref(),
            nivel: nivel.as_deref(),
        };
        // A contagem externa sobrevive ao erro, inclusive depois do POST.
        let mut envio = EnvioHttps;
        let mut contagem = Contador::novo(&mut envio);
        let resultado = executar_criacao(sessao, &pedido, indice, &mut contagem, *confirmar);
        let requisicoes = contagem.feitas();
        let resultado = resultado.map_err(|erro| {
            anyhow::Error::from(erro).context(TentativaDeCriacao { requisicoes })
        })?;
        publicar_resultado(cli.json, &resultado, indice);
        Ok(())
    }))
}

/// Requisições feitas até a falha: depois do POST, a criação pode ter
/// ocorrido. Confira o processo antes de nova confirmação.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
struct TentativaDeCriacao {
    requisicoes: u8,
}

/// Tipo vazio é erro de uso (saída 2), antes de qualquer arquivo.
pub(crate) fn tipo_de_documento(valor: &str) -> Result<String, TipoDeDocumentoVazio> {
    if valor.trim().is_empty() {
        Err(TipoDeDocumentoVazio)
    } else {
        Ok(valor.to_owned())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct TipoDeDocumentoVazio;

fn resultado_json(resultado: &ResultadoCriacao) -> serde_json::Value {
    let numero_sei = match &resultado.estado {
        EstadoCriacao::Previa => None,
        EstadoCriacao::Criado { numero_sei } => Some(numero_sei.as_str()),
    };
    json!({
        "nup": resultado.nup,
        "tipo": resultado.tipo,
        "descricao": resultado.descricao,
        "nivel": resultado.nivel,
        "unidade": resultado.unidade,
        "origem": "web",
        "confirmado": numero_sei.is_some(),
        "alterado": numero_sei.is_some(),
        "numero_sei": numero_sei,
        "requisicoes": resultado.requisicoes
    })
}

fn publicar_resultado(
    json_mode: bool,
    resultado: &ResultadoCriacao,
    indice: &crate::indice_local::IndiceLocal,
) {
    let numero_sei = match &resultado.estado {
        EstadoCriacao::Previa => None,
        EstadoCriacao::Criado { numero_sei } => Some(numero_sei.as_str()),
    };
    if json_mode {
        let mut valor = resultado_json(resultado);
        crate::indice_local::anexar_avisos(&mut valor, indice);
        println!("{valor}");
        return;
    }
    if let Some(numero_sei) = numero_sei {
        println!(
            "Documento {} criado no processo {} (unidade {}), número SEI {}.",
            resultado.tipo, resultado.nup, resultado.unidade, numero_sei
        );
    } else {
        println!(
            "Prévia: criar documento {} no processo {} (unidade {}).",
            resultado.tipo, resultado.nup, resultado.unidade
        );
        println!("Nenhum documento foi criado.");
    }
    if let Some(descricao) = &resultado.descricao {
        println!("Descrição: {descricao}");
    }
    if let Some(nivel) = &resultado.nivel {
        println!("Nível: {nivel}");
    }
    println!("Requisições: {}", resultado.requisicoes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dado_previa_quando_json_entao_expoe_descricao_nivel_e_nenhuma_escrita() {
        // Dado pedido de criacao com descricao/nivel conferidos no cadastro;
        // Quando publico a previa; Entao o usuario ve o que sera gravado.
        let resultado = ResultadoCriacao {
            nup: "00001.000001/2026-01".to_owned(),
            tipo: "Despacho".to_owned(),
            unidade: "UNIDADE TESTE".to_owned(),
            descricao: Some("Ensaio de despacho".to_owned()),
            nivel: Some("publico".to_owned()),
            requisicoes: 5,
            estado: EstadoCriacao::Previa,
        };
        let saida = resultado_json(&resultado);
        assert_eq!(saida["descricao"], "Ensaio de despacho");
        assert_eq!(saida["nivel"], "publico");
        assert_eq!(saida["confirmado"], false);
        assert_eq!(saida["alterado"], false);
        assert!(saida["numero_sei"].is_null());
        assert_eq!(saida["requisicoes"], 5);
    }

    #[test]
    fn tipo_em_branco_e_erro_de_uso() {
        assert!(tipo_de_documento("  ").is_err());
        assert_eq!(tipo_de_documento("Despacho").unwrap(), "Despacho");
    }
}
