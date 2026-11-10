//! Superfície da criação de processos (#245).
//! Nunca transforma um comando incompleto em escrita remota: a etapa Web
//! só será habilitada depois de observar e testar o cadastro servido.
use crate::processo_criar_fluxo::{ErroCriar, PedidoCriar, criar_com_contagem};
use crate::saida_erro::{erro_fonte_http, json_de_erro_fonte, linha_humana};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, configuracao, erro_fonte, transporte};
use serde_json::json;
#[path = "processo_criar_cli/apresentacao.rs"]
mod apresentacao;
#[path = "processo_criar_cli/coincidencias.rs"]
mod coincidencias;
use apresentacao::informar_sucesso;
use coincidencias::{anexar_coincidencias_json, informar_coincidencias_humanas};
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
enum ErroEntradaCriacao {
    Tipo,
    Especificacao,
    Interessado,
    Hipotese,
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
                "erro": "argumento de criação de processo vazio ou com caractere de controle",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: argumento de criação inválido ({erro:?})");
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
fn json_da_recusa_criadora(
    erro: &ErroCriar,
    mensagem: &str,
    requisicoes: u32,
) -> serde_json::Value {
    let mut valor = json_de_erro_fonte(mensagem, erro.status_http());
    valor["requisicoes"] = json!(requisicoes);
    if let Some((limite, tamanho)) = erro.limite_da_especificacao() {
        valor["limite_da_especificacao"] = json!(limite);
        valor["comprimento_da_especificacao_utf16"] = json!(tamanho);
    }
    if erro.nao_aplicada_comprovada() {
        valor["desfecho"] = json!("nao_aplicada");
        valor["repeticao_segura_com_nova_previa"] = json!(true);
    }
    // Nunca autoriza retry automático, mesmo com validação expressa.
    valor
}

/// Mantém o HTTP da leitura posterior separado do HTTP do POST.
fn anexar_http_releitura_json(
    valor: &mut serde_json::Value,
    http: Option<transporte::StatusHttpRecusado>,
) {
    if let Some(http) = http {
        valor["status_http_releitura"] = json!(http.status);
        valor["mensagem_do_servidor_releitura"] = json!(http.mensagem_do_servidor);
    }
}

fn informar_http_releitura(http: Option<transporte::StatusHttpRecusado>) {
    if let Some(http) = http {
        eprintln!("IES: releitura HTTP {}.", http.status);
        if let Some(mensagem) = http.mensagem_do_servidor {
            eprintln!("IES: mensagem saneada da releitura: {mensagem}");
        }
    }
}

/// Apresenta recusas anteriores à criação, separadas do efeito incerto.
fn erro_antes_da_escrita(json_mode: bool, erro: &ErroCriar, requisicoes: u32) -> ExitCode {
    let mensagem = "a tela do SEI não permitiu concluir o fluxo de criação com segurança";
    if let Some(opcoes) = erro.opcoes_para_escolha() {
        if json_mode {
            let mut valor = json_de_erro_fonte(mensagem, erro.status_http());
            valor["hipoteses_legais_oferecidas"] = json_de_opcoes(opcoes);
            valor["requisicoes"] = json!(requisicoes);
            println!("{valor}");
        } else {
            eprintln!("{}", linha_humana(mensagem, erro.status_http()));
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
        println!("{}", json_da_recusa_criadora(erro, mensagem, requisicoes));
        return ExitCode::from(1);
    }
    if let Some((limite, tamanho)) = erro.limite_da_especificacao() {
        eprintln!("{}", linha_humana(mensagem, erro.status_http()));
        eprintln!(
            "IES: especificação com {tamanho} unidades UTF-16; máximo publicado pelo cadastro: {limite}. Nenhuma escrita foi tentada."
        );
        return ExitCode::from(1);
    }
    if erro.nao_aplicada_comprovada() {
        eprintln!("{}", linha_humana(mensagem, erro.status_http()));
        eprintln!(
            "IES: o SEI recusou expressamente o cadastro; criação não aplicada. Para tentar de novo, faça nova prévia e nova confirmação explícita (sem retry automático)."
        );
        return ExitCode::from(1);
    }
    erro_fonte_http(false, mensagem, erro.status_http())
}

fn erro_do_fluxo(json_mode: bool, erro: &ErroCriar, requisicoes: u32) -> ExitCode {
    if !json_mode {
        eprintln!("IES: {requisicoes} requisições efetivamente tentadas neste fluxo.");
    }
    if !erro.apos_post() {
        return erro_antes_da_escrita(json_mode, erro, requisicoes);
    }
    // O GET de reconciliação é só observação: sem conferir tipo e
    // especificação de cada NUP, jamais afirmar "não aplicada" ou habilitar retry.
    if json_mode {
        let mut valor = json!({
            "codigo": "nao_confirmado",
            "erro": "ainda não foi possível confirmar o efeito da criação; não repita o POST",
            "retentavel": false,
            "requisicoes": requisicoes
        });
        if let Some(diagnostico) = erro.diagnostico_apos_post() {
            let causa_post = diagnostico.causa_post;
            let falha_leitura = diagnostico.causa_releitura;
            let candidatos = diagnostico.candidatos_observados;
            valor["causa_post"] = json!(causa_post);
            valor["causa_releitura"] = json!(falha_leitura);
            valor["candidatos_observados_na_caixa"] = json!(candidatos);
            anexar_http_releitura_json(&mut valor, diagnostico.http_releitura);
            if let Some(nups) = diagnostico.candidatos_compativeis
                && let Some(evidencia) = diagnostico.evidencia
            {
                anexar_coincidencias_json(&mut valor, evidencia, &nups);
            }
        }
        if let Some(http) = erro.status_http() {
            valor["status_http"] = json!(http.status);
            valor["mensagem_do_servidor"] = json!(http.mensagem_do_servidor);
        }
        println!("{valor}");
    } else {
        // A linha humana preserva o HTTP saneado também na saída 3.
        let mensagem = "criação não confirmada; não repita o POST";
        eprintln!("{}", linha_humana(mensagem, erro.status_http()));
        if let Some(diagnostico) = erro.diagnostico_apos_post() {
            let causa_post = diagnostico.causa_post;
            let falha_leitura = diagnostico.causa_releitura;
            let candidatos = diagnostico.candidatos_observados;
            eprintln!("IES: causa do POST: {causa_post}");
            if let Some(causa) = falha_leitura {
                eprintln!("IES: causa da releitura: {causa}");
            }
            informar_http_releitura(diagnostico.http_releitura);
            if let Some(nups) = candidatos {
                eprintln!(
                    "IES: candidatos observados na caixa Gerados: {}",
                    nups.len()
                );
                for nup in nups {
                    eprintln!("  {nup}");
                }
            }
            if let Some(nups) = diagnostico.candidatos_compativeis
                && let Some(evidencia) = diagnostico.evidencia
            {
                informar_coincidencias_humanas(&nups, evidencia);
            }
        }
        eprintln!("IES: confira a caixa Gerados antes de tentar novamente.");
    }
    ExitCode::from(1)
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
        Err(erro) => return Some(erro_fonte(cli.json, erro.mensagem())),
    };
    let referencias = match transporte::sessao::ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    ) {
        Ok(referencias) => referencias,
        Err(erro) => return Some(erro_fonte(cli.json, erro.mensagem())),
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
    let mut requisicoes = 0;
    let resultado = criar_com_contagem(
        referencias.origem(),
        referencias.url_do_controle(),
        referencias.cookie(),
        &pedido,
        &mut EnvioHttps,
        &mut requisicoes,
    );
    Some(match resultado {
        Ok(feito) => informar_sucesso(cli.json, feito, especificacao),
        Err(erro) => erro_do_fluxo(cli.json, &erro, requisicoes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processo_criar_fluxo::EvidenciaDeCoincidencia;

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
    fn erro_de_maxlength_publica_limite_observado_sem_texto_da_especificacao() {
        let erro = ErroCriar::Cadastro(
            crate::processo_criar_cadastro::ErroCadastro::EspecificacaoAcimaDoLimite {
                limite: 100,
                tamanho: 102,
            },
        );
        let json = json_da_recusa_criadora(&erro, "Limite do cadastro", 3);
        assert_eq!(json["limite_da_especificacao"], 100);
        assert_eq!(json["comprimento_da_especificacao_utf16"], 102);
        assert_eq!(json["requisicoes"], 3);
        assert_eq!(json["retentavel"], false);
        assert!(json.get("desfecho").is_none());
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
        let json = json_da_recusa_criadora(&recusada, "Recusa reconhecida", 4);
        assert_eq!(json["desfecho"], "nao_aplicada");
        assert_eq!(json["requisicoes"], 4);
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
        let json_http = json_da_recusa_criadora(&http, "Recusa HTTP", 4);
        assert!(json_http.get("desfecho").is_none());
        assert!(json_http.get("repeticao_segura_com_nova_previa").is_none());
        assert_eq!(json_http["status_http"], 403);
        assert_eq!(json_http["requisicoes"], 4);
    }

    #[test]
    fn json_separa_os_status_http_do_post_e_da_releitura() {
        let mut saida = json!({
            "status_http": 503,
            "mensagem_do_servidor": null
        });
        anexar_http_releitura_json(
            &mut saida,
            Some(transporte::StatusHttpRecusado {
                status: 502,
                mensagem_do_servidor: Some("Falha de leitura saneada".to_owned()),
            }),
        );
        assert_eq!(saida["status_http"], 503); // status anterior, do POST
        assert_eq!(saida["status_http_releitura"], 502); // status do GET
        assert_eq!(
            saida["mensagem_do_servidor_releitura"],
            "Falha de leitura saneada"
        );
        let mut sem_http = json!({"causa_releitura": "falha_de_rede"});
        anexar_http_releitura_json(&mut sem_http, None);
        assert!(sem_http.get("status_http_releitura").is_none());
    }

    #[test]
    fn coincidencia_parcial_no_json_nao_vira_candidatos_ambiguos() {
        let mut parcial = json!({"codigo": "nao_confirmado"});
        anexar_coincidencias_json(
            &mut parcial,
            EvidenciaDeCoincidencia::Parcial,
            &["NUP-sintetico-10".into()],
        );
        assert_eq!(
            parcial["coincidencias_parciais_conferidas"],
            json!(["NUP-sintetico-10"])
        );
        assert!(parcial.get("desfecho").is_none());
        assert!(parcial.get("candidatos_compativeis").is_none());

        let mut completa = json!({"codigo": "nao_confirmado"});
        anexar_coincidencias_json(
            &mut completa,
            EvidenciaDeCoincidencia::Multiplas,
            &["NUP-sintetico-10".into(), "NUP-sintetico-11".into()],
        );
        assert_eq!(completa["desfecho"], "candidatos_ambiguos");
        assert_eq!(
            completa["candidatos_compativeis"],
            json!(["NUP-sintetico-10", "NUP-sintetico-11"])
        );
        assert!(completa.get("coincidencias_parciais_conferidas").is_none());
        let mut unica = json!({"codigo": "nao_confirmado"});
        anexar_coincidencias_json(
            &mut unica,
            EvidenciaDeCoincidencia::UnicaSemProvaCausal,
            &["NUP-sintetico-10".into()],
        );
        assert_eq!(unica["desfecho"], "possivel_residuo");
        assert_eq!(unica["candidatos_compativeis"], json!(["NUP-sintetico-10"]));
        assert!(unica.get("coincidencias_parciais_conferidas").is_none());
    }

    #[test]
    fn entrada_valida_nao_implica_autorizacao_de_escrita() {
        assert_eq!(
            validar("Início de Processo", "Ensaio controlado", &[]),
            Ok(())
        );
    }
}
