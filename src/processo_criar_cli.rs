//! Superfície da criação de processos (#245).
//! Nunca transforma um comando incompleto em escrita remota: a etapa Web
//! só será habilitada depois de observar e testar o cadastro servido.
use crate::processo_criar_fluxo::{ErroCriar, PedidoCriar, criar};
use crate::saida_erro::{erro_fonte_http, json_de_erro_fonte, linha_humana};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, configuracao, erro_fonte, transporte};
use serde_json::json;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
enum ErroEntradaCriacao {
    #[causa(nome = "tipo_de_processo_invalido")]
    Tipo,
    #[causa(nome = "especificacao_de_processo_invalida")]
    Especificacao,
    #[causa(nome = "interessado_de_processo_invalido")]
    Interessado,
    #[causa(nome = "hipotese_legal_informada_invalida")]
    Hipotese,
}

impl ErroEntradaCriacao {
    fn subcodigo(self) -> &'static str {
        crate::erro_publico::folha(&self)
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

/// Construção pura para testar que a recusa do próprio cadastro distingue
/// não aplicação comprovada de erro HTTP sem garantia de não aplicação.
fn json_da_recusa_criadora(erro: &ErroCriar, mensagem: &str) -> serde_json::Value {
    let mut valor = json_de_erro_fonte(erro.subcodigo(), mensagem, erro.status_http());
    if erro.nao_aplicada_comprovada() {
        valor["desfecho"] = json!("nao_aplicada");
        valor["repeticao_segura_com_nova_previa"] = json!(true);
    }
    // Nunca autoriza retry automático, mesmo com validação expressa.
    valor
}

fn erro_do_fluxo(json_mode: bool, erro: &ErroCriar) -> ExitCode {
    if erro.saida() != 3 {
        let mensagem = "a tela do SEI não permitiu concluir o fluxo de criação com segurança";
        if let Some(opcoes) = erro.opcoes_para_escolha() {
            if json_mode {
                let mut valor = json_de_erro_fonte(erro.subcodigo(), mensagem, erro.status_http());
                valor["hipoteses_legais_oferecidas"] = json_de_opcoes(opcoes);
                println!("{valor}");
            } else {
                eprintln!(
                    "{}",
                    linha_humana(erro.subcodigo(), mensagem, erro.status_http())
                );
                eprintln!("IES: hipóteses legais ofertadas no AJAX:");
                for (id, nome) in opcoes {
                    eprintln!("  {id}#{nome}");
                }
                eprintln!("IES: faça nova prévia com --hipotese-legal ID#NOME antes de confirmar.");
            }
            return ExitCode::from(1);
        }
        // Status e mensagem saneada do SEI vão para a saída quando houve.
        if json_mode {
            println!("{}", json_da_recusa_criadora(erro, mensagem));
            return ExitCode::from(1);
        }
        if erro.nao_aplicada_comprovada() {
            eprintln!(
                "{}",
                linha_humana(erro.subcodigo(), mensagem, erro.status_http())
            );
            eprintln!(
                "IES: o SEI recusou expressamente o cadastro; criação não aplicada. Para tentar de novo, faça nova prévia e nova confirmação explícita (sem retry automático)."
            );
            return ExitCode::from(1);
        }
        return erro_fonte_http(false, erro.subcodigo(), mensagem, erro.status_http());
    }
    // O GET de reconciliação é só observação: sem conferir tipo e
    // especificação de cada NUP, jamais afirmar "não aplicada" ou habilitar retry.
    if json_mode {
        let mut valor = json!({
            "codigo": "efeito_incerto",
            "subcodigo": erro.subcodigo(),
            "erro": "ainda não foi possível confirmar o efeito da criação; não repita o POST",
            "retentavel": false
        });
        if let Some(diagnostico) = erro.diagnostico_apos_post() {
            let causa_post = diagnostico.causa_post;
            let falha_leitura = diagnostico.causa_releitura;
            let candidatos = diagnostico.candidatos_observados;
            valor["causa_post"] = json!(causa_post);
            valor["causa_releitura"] = json!(falha_leitura);
            valor["candidatos_observados_na_caixa"] = json!(candidatos);
            if let Some(nups) = diagnostico.candidatos_compativeis {
                valor["desfecho"] = json!("candidatos_ambiguos");
                valor["candidatos_compativeis"] = json!(nups);
            }
        }
        if let Some(http) = erro.status_http() {
            valor["status_http"] = json!(http.status);
            valor["mensagem_do_servidor"] = json!(http.mensagem_do_servidor);
        }
        println!("{valor}");
    } else {
        // O 4xx isolado não prova ausência de efeito, mas seu status e
        // mensagem saneada continuam disponíveis para diagnóstico.
        if erro.status_http().is_some() {
            eprintln!(
                "{}",
                linha_humana(
                    erro.subcodigo(),
                    "criação não confirmada; não repita o POST",
                    erro.status_http(),
                )
            );
        } else {
            eprintln!("IES: criação não confirmada [{}]", erro.subcodigo());
        }
        if let Some(diagnostico) = erro.diagnostico_apos_post() {
            let causa_post = diagnostico.causa_post;
            let falha_leitura = diagnostico.causa_releitura;
            let candidatos = diagnostico.candidatos_observados;
            eprintln!("IES: causa do POST: {causa_post}");
            if let Some(causa) = falha_leitura {
                eprintln!("IES: causa da releitura: {causa}");
            }
            if let Some(nups) = candidatos {
                eprintln!(
                    "IES: candidatos observados na caixa Gerados: {}",
                    nups.len()
                );
                for nup in nups {
                    eprintln!("  {nup}");
                }
            }
            if let Some(nups) = diagnostico.candidatos_compativeis {
                eprintln!(
                    "IES: {} candidatos com identidade, tipo e especificação coincidentes; autoria da criação ambígua.",
                    nups.len()
                );
                for nup in nups {
                    eprintln!("  {nup}");
                }
            }
        }
        eprintln!("IES: confira a caixa Gerados antes de tentar novamente.");
    }
    ExitCode::from(3)
}

fn informar_causa_post(causa: Option<&str>) {
    if let Some(causa) = causa {
        println!("Causa do POST reconciliado por leitura: {causa}");
    }
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
                println!("Acesso: {}", feito.nivel);
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
    fn recusa_explicita_publica_nao_aplicada_sem_inferir_de_403() {
        use crate::formulario_web::ErroSubmissaoFormulario;
        use crate::transporte::StatusHttpRecusado;
        let recusada = ErroCriar::RecusaDoPost(ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(
            StatusHttpRecusado {
                status: 200,
                mensagem_do_servidor: Some("Cadastro recusado".to_owned()),
            },
        ));
        assert_eq!(recusada.saida(), 1);
        let json = json_da_recusa_criadora(&recusada, "Recusa reconhecida");
        assert_eq!(json["desfecho"], "nao_aplicada");
        assert_eq!(json["repeticao_segura_com_nova_previa"], true);
        assert_eq!(json["retentavel"], false); // nunca retry automatico
        assert_eq!(json["mensagem_do_servidor"], "Cadastro recusado");
        assert!(json.get("status_http").is_none()); // validacao da Infra foi HTTP 200

        // Um 403 pode vir de intermediário: não confundir com prova do SEI.
        let http = ErroCriar::RespostaHttpSemProva(ErroSubmissaoFormulario::Transporte(
            crate::transporte::FalhaTransporte::AcessoNegado(StatusHttpRecusado {
                status: 403,
                mensagem_do_servidor: Some("Proibido".to_owned()),
            }),
        ));
        assert_eq!(http.saida(), 3); // não autorizar repetição após 403
        let json_http = json_da_recusa_criadora(&http, "Recusa HTTP");
        assert!(json_http.get("desfecho").is_none());
        assert!(json_http.get("repeticao_segura_com_nova_previa").is_none());
        assert_eq!(json_http["status_http"], 403);
    }

    #[test]
    fn entrada_valida_nao_implica_autorizacao_de_escrita() {
        assert_eq!(
            validar("Início de Processo", "Ensaio controlado", &[]),
            Ok(())
        );
    }
}
