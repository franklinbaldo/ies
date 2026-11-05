//! Superfície da criação de processos (#245).
//! Nunca transforma um comando incompleto em escrita remota: a etapa Web
//! só será habilitada depois de observar e testar o cadastro servido.
use crate::processo_criar_fluxo::{ErroCriar, PedidoCriar, criar};
use crate::saida_erro::{erro_fonte_http, json_de_erro_fonte, linha_humana};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, configuracao, erro_fonte, transporte};
use serde_json::json;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroEntradaCriacao {
    Tipo,
    Especificacao,
    Interessado,
    Hipotese,
}

impl ErroEntradaCriacao {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::Tipo => "tipo_de_processo_invalido",
            Self::Especificacao => "especificacao_de_processo_invalida",
            Self::Interessado => "interessado_de_processo_invalido",
            Self::Hipotese => "hipotese_legal_informada_invalida",
        }
    }
}

fn texto_valido(valor: &str) -> bool {
    !valor.trim().is_empty() && !valor.chars().any(char::is_control)
}

fn validar(
    tipo: &str,
    especificacao: &str,
    interessados: &[String],
) -> Result<(), ErroEntradaCriacao> {
    if !texto_valido(tipo) {
        return Err(ErroEntradaCriacao::Tipo);
    }
    if !texto_valido(especificacao) {
        return Err(ErroEntradaCriacao::Especificacao);
    }
    if interessados.iter().any(|v| !texto_valido(v)) {
        return Err(ErroEntradaCriacao::Interessado);
    }
    Ok(())
}

/// A seleção explícita é sempre restrita e identifica ID e nome observados.
fn validar_hipotese(nivel: &str, escolha: Option<&str>) -> Result<(), ErroEntradaCriacao> {
    let Some(escolha) = escolha else {
        return Ok(());
    };
    let Some((id, nome)) = escolha.split_once('#') else {
        return Err(ErroEntradaCriacao::Hipotese);
    };
    if nivel != "restrito"
        || !texto_valido(id)
        || !texto_valido(nome)
        || id != id.trim()
        || nome != nome.trim()
    {
        return Err(ErroEntradaCriacao::Hipotese);
    }
    Ok(())
}

fn uso_invalido(json_mode: bool, erro: ErroEntradaCriacao) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": erro.subcodigo(),
                "erro": "argumento de criação de processo vazio ou com caractere de controle",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: argumento de criação inválido [{}]", erro.subcodigo());
    }
    ExitCode::from(2)
}

/// O diagnóstico é formado exclusivamente por opções XML habilitadas,
/// de IDs distintos e sem controles. Nada de URLs assinadas ou cookies.
fn json_de_opcoes(opcoes: &[(String, String)]) -> serde_json::Value {
    json!(
        opcoes
            .iter()
            .map(|(id, nome)| {
                json!({ "id": id, "nome": nome, "argumento": format!("{id}#{nome}") })
            })
            .collect::<Vec<_>>()
    )
}

fn erro_do_fluxo(json_mode: bool, erro: &ErroCriar) -> ExitCode {
    if erro.saida() != 3 {
        let mensagem = "a tela do SEI não permitiu concluir o fluxo de criação com segurança";
        if let Some(opcoes) = erro.opcoes_para_escolha() {
            if json_mode {
                let mut valor = json_de_erro_fonte(
                    erro.subcodigo(),
                    mensagem,
                    erro.status_http(),
                );
                valor["hipoteses_legais_oferecidas"] = json_de_opcoes(opcoes);
                println!("{valor}");
            } else {
                eprintln!("{}", linha_humana(erro.subcodigo(), mensagem, erro.status_http()));
                eprintln!("IES: hipóteses legais ofertadas no AJAX:");
                for (id, nome) in opcoes {
                    eprintln!("  {id}#{nome}");
                }
                eprintln!("IES: faça nova prévia com --hipotese-legal ID#NOME antes de confirmar.");
            }
            return ExitCode::from(1);
        }
        // Status e mensagem saneada do SEI vão para a saída quando houve.
        return erro_fonte_http(
            json_mode,
            erro.subcodigo(),
            mensagem,
            erro.status_http(),
        );
    }
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "efeito_incerto",
                "subcodigo": erro.subcodigo(),
                "erro": "o envio pode ter criado o processo; confira os gerados antes de repetir",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: criação não confirmada [{}]", erro.subcodigo());
        eprintln!("IES: confira a caixa Gerados antes de tentar novamente.");
    }
    ExitCode::from(3)
}

/// Valida os argumentos antes de acessar os arquivos privados da sessão.
/// Nenhum retry: o fluxo remoto consome cada resposta exatamente uma vez.
pub(crate) fn tentar_criar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Criar {
                tipo,
                especificacao,
                interessado,
                nivel,
                hipotese_legal,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    if let Err(erro) = validar(tipo, especificacao, interessado)
        .and_then(|()| validar_hipotese(nivel, hipotese_legal.as_deref()))
    {
        return Some(uso_invalido(cli.json, erro));
    }
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return Some(erro_fonte(cli.json, erro.subcodigo(), erro.mensagem())),
    };
    let referencias = match transporte::sessao::ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    ) {
        Ok(referencias) => referencias,
        Err(erro) => return Some(erro_fonte(cli.json, erro.subcodigo(), erro.mensagem())),
    };
    let pedido = PedidoCriar {
        tipo,
        especificacao,
        interessados: interessado,
        nivel,
        unidade: cli.unidade.as_deref(),
        hipotese_legal: hipotese_legal.as_deref(),
        confirmar: *confirmar,
    };
    let resultado = criar(
        referencias.origem(),
        referencias.url_do_controle(),
        referencias.cookie(),
        &pedido,
        &mut EnvioHttps,
    );
    Some(match resultado {
        Ok(feito) => {
            if cli.json {
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
                println!("Acesso: {}", feito.nivel);
                if let Some(hipotese) = feito.hipotese_legal {
                    println!("Hipótese legal observada (ID): {hipotese}");
                }
                if let Some(nup) = feito.nup {
                    println!("NUP criado e verificado: {nup}");
                } else {
                    println!("Para criar, repita com --confirmar.");
                }
            }
            ExitCode::SUCCESS
        }
        Err(erro) => erro_do_fluxo(cli.json, &erro),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entrada_invalida_eh_recusada_sem_sessao() {
        assert_eq!(validar(" ", "Ensaio", &[]), Err(ErroEntradaCriacao::Tipo));
        assert_eq!(
            validar("Tipo", "\n", &[]),
            Err(ErroEntradaCriacao::Especificacao)
        );
        assert_eq!(
            validar("Tipo", "Ensaio", &["\t".to_owned()]),
            Err(ErroEntradaCriacao::Interessado)
        );
    }

    #[test]
    fn hipotese_legal_exige_restrito_e_identificacao_exata() {
        assert_eq!(
            validar_hipotese("publico", Some("31#Base A")),
            Err(ErroEntradaCriacao::Hipotese)
        );
        assert_eq!(
            validar_hipotese("restrito", Some("31")),
            Err(ErroEntradaCriacao::Hipotese)
        );
        assert_eq!(validar_hipotese("restrito", Some("31#Base A")), Ok(()));
    }

    #[test]
    fn diagnostico_json_da_hipotese_nao_expoe_url_assinada() {
        let opcoes = vec![
            ("31".to_owned(), "Base A".to_owned()),
            ("32".to_owned(), "Base B".to_owned()),
        ];
        assert_eq!(
            json_de_opcoes(&opcoes),
            json!([
                { "id": "31", "nome": "Base A", "argumento": "31#Base A" },
                { "id": "32", "nome": "Base B", "argumento": "32#Base B" }
            ])
        );
    }

    #[test]
    fn entrada_valida_nao_implica_autorizacao_de_escrita() {
        assert_eq!(
            validar("Início de Processo", "Ensaio controlado", &[]),
            Ok(())
        );
    }
}
