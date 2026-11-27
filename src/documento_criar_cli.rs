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

#[path = "documento_criar/copia_editavel.rs"]
mod copia_editavel;
use copia_editavel::CopiaEditavel;
#[cfg(test)]
pub(crate) use copia_editavel::contem_estilos;

pub(crate) fn tentar_criar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::Criar {
                processo,
                tipo,
                descricao,
                nivel,
                texto_padrao,
                modelo,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    // O nível de escrita é pré-condição independente da sessão: a recusa
    // vem antes de abrir arquivos privados.
    if *confirmar && let Err(erro) = validar_nivel_de_confirmacao(*nivel) {
        return Some(Err(erro.into()));
    }
    if (texto_padrao.is_some() || modelo.is_some())
        && let Err(erro) = conferir_origem_da_previa()
    {
        return Some(Err(erro));
    }
    Some(com_sessao(cli, |sessao, indice| {
        let pasta = if *confirmar {
            let pasta = std::env::current_dir()?;
            crate::foto_documento::conferir_pasta(&pasta)?;
            Some(pasta)
        } else {
            None
        };
        let pedido = PedidoCriacao {
            nup: processo,
            unidade: cli.unidade.as_deref(),
            tipo,
            descricao: descricao.as_deref(),
            nivel: *nivel,
            texto_padrao: texto_padrao.as_deref(),
            modelo: modelo.as_ref(),
        };
        // A contagem externa sobrevive ao erro, inclusive depois do POST.
        let mut envio = EnvioHttps;
        let mut contagem = Contador::novo(&mut envio);
        let resultado = executar_criacao(sessao, &pedido, indice, &mut contagem, *confirmar);
        let requisicoes = contagem.feitas();
        let mut resultado = resultado.map_err(|erro| {
            anyhow::Error::from(erro).context(TentativaDeCriacao { requisicoes })
        })?;
        let copia = match (&resultado.estado, pasta) {
            (EstadoCriacao::Criado { numero_sei }, Some(pasta)) => {
                copia_editavel::preparar(&pasta, numero_sei, &pedido, sessao, indice, &mut contagem)
                    .map(Some)
                    .map_err(|erro| {
                        anyhow::Error::from(erro).context(CriadoSemCopia {
                            numero_sei: numero_sei.clone(),
                        })
                    })
            }
            _ => Ok(None),
        };
        resultado.requisicoes = contagem.feitas();
        publicar_resultado(
            cli.json,
            &resultado,
            indice,
            copia.as_ref().ok().and_then(Option::as_ref),
        );
        copia.map(|_| ())
    }))
}

// A hipótese experimental é restrita antes de ler a sessão. O núcleo
// confere novamente a origem efetiva, inclusive após troca de sessão.
fn conferir_origem_da_previa() -> anyhow::Result<()> {
    let arquivos = crate::configuracao::arquivos_de_acesso()?;
    let origem = crate::arquivo_privado::ler(&arquivos.origem, 512)
        .map_err(crate::transporte::FalhaTransporte::OrigemIlegivel)?;
    crate::origem_de_treinamento::exigir_origem(origem.trim())?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Documento SEI {numero_sei} criado e confirmado. A cópia editável falhou; recupere o documento pelo número SEI."
)]
struct CriadoSemCopia {
    numero_sei: String,
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
    let mut saida = json!({
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
    });
    if let (Some(texto), Some(objeto)) = (&resultado.texto_padrao, saida.as_object_mut()) {
        objeto.insert("texto_padrao".to_owned(), json!(texto));
    }
    if let (Some(modelo), Some(objeto)) = (&resultado.modelo, saida.as_object_mut()) {
        objeto.insert("modelo_solicitado".to_owned(), json!(modelo));
    }
    saida
}

fn publicar_resultado(
    json_mode: bool,
    resultado: &ResultadoCriacao,
    indice: &crate::indice_local::IndiceLocal,
    copia: Option<&CopiaEditavel>,
) {
    let numero_sei = match &resultado.estado {
        EstadoCriacao::Previa => None,
        EstadoCriacao::Criado { numero_sei } => Some(numero_sei.as_str()),
    };
    if json_mode {
        let mut valor = resultado_json(resultado);
        if let (Some(copia), Some(objeto)) = (copia, valor.as_object_mut()) {
            objeto.insert("copia_editavel".to_owned(), json!(copia));
        }
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
    if let Some(nivel) = resultado.nivel {
        println!("Nível: {}", nivel.como_texto());
    }
    if let Some(texto) = &resultado.texto_padrao {
        println!("Texto padrão: {} (ID {})", texto.nome, texto.id);
    }
    if let Some(modelo) = &resultado.modelo {
        println!("Modelo solicitado: {modelo} (identidade e conteúdo não conferidos).");
    }
    if let Some(copia) = copia {
        println!("Cópia editável: {}", copia.arquivo);
        if let Some(orientacao) = copia.orientacao {
            println!("{orientacao}");
        }
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
            nivel: Some(crate::dominio::NivelDeAcesso::Publico),
            requisicoes: 5,
            texto_padrao: None,
            modelo: None,
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
