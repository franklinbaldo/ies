//! Entrada pública de documento criar (#246).
//! A confirmação deriva um único POST do cadastro e só reconhece o efeito
//! depois de releitura da árvore. Não há retry.
use crate::documento_criar::{
    ErroPreviaCriacao, EstadoCriacao, PedidoCriacao, ResultadoCriacao, executar_criacao,
    validar_nivel_de_confirmacao,
};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::Contador;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_criar(cli: &Cli) -> Option<ExitCode> {
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
    let nup = processo.clone();
    if tipo.trim().is_empty() {
        return Some(entrada_invalida(cli.json));
    }
    // O nivel de escrita ja e uma pre-condicao independente da sessao.
    // A CLI deve informar essa causa antes de abrir arquivos privados.
    if *confirmar && let Err(erro) = validar_nivel_de_confirmacao(nivel.as_deref()) {
        return Some(publicar_erro(cli.json, &erro, 0, &nup));
    }
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoCriacao {
            nup: &nup,
            unidade: cli.unidade.as_deref(),
            tipo,
            descricao: descricao.as_deref(),
            nivel: nivel.as_deref(),
        };
        // A contagem externa sobrevive ao Result::Err, inclusive depois do POST.
        let mut envio = EnvioHttps;
        let mut contagem = Contador::novo(&mut envio);
        match executar_criacao(sessao, &pedido, indice, &mut contagem, *confirmar) {
            Ok(resultado) => publicar_resultado(cli.json, &resultado),
            Err(erro) => publicar_erro(cli.json, &erro, contagem.feitas(), &nup),
        }
    }))
}

fn entrada_invalida(json_mode: bool) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "erro": "parâmetro de criação de documento inválido",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: parâmetro de criação de documento inválido");
    }
    ExitCode::from(2)
}

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

fn publicar_resultado(json_mode: bool, resultado: &ResultadoCriacao) -> ExitCode {
    let numero_sei = match &resultado.estado {
        EstadoCriacao::Previa => None,
        EstadoCriacao::Criado { numero_sei } => Some(numero_sei.as_str()),
    };
    if json_mode {
        println!("{}", resultado_json(resultado));
    } else {
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
    ExitCode::SUCCESS
}

const MENSAGEM_INCERTEZA: &str = "Criação de documento não comprovada. Confira o processo antes de qualquer nova confirmação; não repita --confirmar sem verificar.";

fn erro_incerto_json(
    erro: &ErroPreviaCriacao,
    requisicoes: u8,
    nup: &NumeroProcesso,
) -> serde_json::Value {
    let mut saida = json!({
        "codigo": "nao_confirmado",
        "erro": MENSAGEM_INCERTEZA,
        "verificar_com": format!("ies processo {}", nup.as_str()),
        "retentavel": false,
        "requisicoes": requisicoes
    });
    if let ErroPreviaCriacao::NaoConfirmadoNaReleitura(causa) = erro {
        saida["causa_releitura"] = json!(causa.mensagem());
    }
    if let Some(http) = erro.status_http() {
        saida["status_http"] = json!(http.status);
        if let Some(mensagem) = &http.mensagem_do_servidor {
            saida["mensagem_do_servidor"] = json!(mensagem);
        }
    }
    saida
}

/// Mensagem orientada à causa observada, sem classificar por texto.
fn mensagem_da_recusa(erro: &ErroPreviaCriacao) -> &'static str {
    use crate::documento_criar::ErroCriar;
    match erro {
        ErroPreviaCriacao::Tipo(ErroCriar::TipoAusente) => {
            "tipo de documento não oferecido pelo SEI"
        }
        ErroPreviaCriacao::Tipo(ErroCriar::TipoAmbiguo) => {
            "mais de um tipo de documento com o nome solicitado"
        }
        ErroPreviaCriacao::ProcessoNaoAbertoNaUnidade => {
            "processo não está aberto na unidade da sessão"
        }
        ErroPreviaCriacao::NivelNaoInformado => "informe --nivel publico para confirmar a criação",
        ErroPreviaCriacao::NivelRestritoSemHipotese => {
            "nível não público exige hipótese legal não disponível nesta operação"
        }
        ErroPreviaCriacao::SubmissaoNaoComprovada => {
            "o SEI não apresentou uma forma reconhecida de concluir a criação"
        }
        ErroPreviaCriacao::ArvoreIncompleta => {
            "árvore do processo contém pastas não carregadas; criação não pode ser comprovada"
        }
        ErroPreviaCriacao::FotoInicialInconsistente => {
            "árvore inicial não permite distinguir os documentos com segurança"
        }
        _ => "criação do documento não pôde ser conferida",
    }
}

fn publicar_erro(
    json_mode: bool,
    erro: &ErroPreviaCriacao,
    requisicoes: u8,
    nup: &NumeroProcesso,
) -> ExitCode {
    if matches!(
        erro,
        ErroPreviaCriacao::NaoConfirmado
            | ErroPreviaCriacao::NaoConfirmadoNoEnvio(_)
            | ErroPreviaCriacao::NaoConfirmadoNaReleitura(_)
    ) {
        if json_mode {
            println!("{}", erro_incerto_json(erro, requisicoes, nup));
        } else {
            eprintln!("IES: {MENSAGEM_INCERTEZA}");
            eprintln!("Verifique com: ies processo {}", nup.as_str());
            eprintln!("Requisições: {requisicoes}");
            if let Some(causa) = erro.causa_envio() {
                eprintln!(
                    "{}",
                    saida_erro::linha_humana(
                        "falha observada no envio do documento",
                        causa.status_http(),
                    )
                );
            }
            if let ErroPreviaCriacao::NaoConfirmadoNaReleitura(causa) = erro {
                eprintln!(
                    "{}",
                    saida_erro::linha_humana(
                        "falha observada na releitura após envio",
                        causa.status_http(),
                    )
                );
            }
        }
        return ExitCode::from(1);
    }
    saida_erro::erro_fonte_http(json_mode, mensagem_da_recusa(erro), erro.status_http())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nup_de_teste() -> NumeroProcesso {
        NumeroProcesso::novo("00001.000001/2026-01").unwrap()
    }

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
    fn nao_confirmado_preserva_status_200_da_validacao_infra() {
        use crate::formulario_web::ErroSubmissaoFormulario;
        use crate::transporte::StatusHttpRecusado;

        // HTTP 200 de validação é uma recusa real, não ausência de status.
        // Sem rede: a fronteira só apresenta a causa do único envio tentado.
        let erro = ErroPreviaCriacao::NaoConfirmadoNoEnvio(
            ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(StatusHttpRecusado {
                status: 200,
                mensagem_do_servidor: Some("Descrição obrigatória".to_owned()),
            }),
        );
        let saida = erro_incerto_json(&erro, 6, &nup_de_teste());
        assert_eq!(saida["codigo"], "nao_confirmado");
        assert_eq!(saida["status_http"], 200);
        assert_eq!(saida["mensagem_do_servidor"], "Descrição obrigatória");
        assert_eq!(saida["requisicoes"], 6);
        assert_eq!(saida["retentavel"], false);
        assert_eq!(
            publicar_erro(false, &erro, 6, &nup_de_teste()),
            ExitCode::from(1)
        );
    }

    #[test]
    fn nao_confirmado_publica_requisicoes_realmente_tentadas() {
        let saida = erro_incerto_json(&ErroPreviaCriacao::NaoConfirmado, 7, &nup_de_teste());
        assert_eq!(saida["codigo"], "nao_confirmado");
        assert_eq!(
            saida["requisicoes"], 7,
            "null não informa a contagem já observada"
        );
        assert_eq!(saida["retentavel"], false);
    }

    #[test]
    fn nao_confirmado_da_releitura_preserva_http_e_contagem_sem_retry() {
        use crate::processo_obter::ErroProcessoObter;
        use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

        let erro = ErroPreviaCriacao::NaoConfirmadoNaReleitura(ErroProcessoObter::Transporte(
            FalhaTransporte::AcessoNegado(StatusHttpRecusado {
                status: 403,
                mensagem_do_servidor: Some("leitura recusada".to_owned()),
            }),
        ));
        let saida = erro_incerto_json(&erro, 7, &nup_de_teste());
        assert_eq!(saida["status_http"], 403);
        assert_eq!(saida["mensagem_do_servidor"], "leitura recusada");
        assert_eq!(saida["requisicoes"], 7);
        assert!(saida["causa_releitura"].is_string());
        assert_eq!(saida["retentavel"], false);
        assert_eq!(
            publicar_erro(false, &erro, 7, &nup_de_teste()),
            ExitCode::from(1)
        );
    }
    #[test]
    fn nao_confirmado_orienta_a_conferir_o_processo_antes_de_outra_criacao() {
        // Teste auxiliar do BDD-246-04: efeito incerto exige leitura independente.
        let erro = ErroPreviaCriacao::NaoConfirmado;
        let saida = erro_incerto_json(&erro, 7, &nup_de_teste());
        assert_eq!(saida["codigo"], "nao_confirmado");
        assert_eq!(saida["verificar_com"], "ies processo 00001.000001/2026-01");
        assert!(
            saida["erro"]
                .as_str()
                .is_some_and(|mensagem| mensagem.contains("Confira o processo"))
        );
        assert_eq!(saida["retentavel"], false);
        assert_eq!(
            publicar_erro(false, &erro, 7, &nup_de_teste()),
            ExitCode::from(1)
        );
    }

    #[test]
    fn bdd_246_04_nao_confirmado_indica_nup_real_na_releitura() {
        // Dado um efeito incerto ao incluir documento em um processo conhecido;
        // Quando a CLI apresenta o erro; Então oferece comando de leitura copiável.
        let nup = NumeroProcesso::novo("00001.000001/2026-01").unwrap();
        let erro = ErroPreviaCriacao::NaoConfirmado;
        let saida = erro_incerto_json(&erro, 7, &nup);
        assert_eq!(saida["verificar_com"], "ies processo 00001.000001/2026-01");
        assert_eq!(saida["retentavel"], false);
        assert_eq!(saida["codigo"], "nao_confirmado");
    }

    #[test]
    fn bdd_246_03_recusa_da_cli_explica_tipo_nao_oferecido_ou_ambiguo() {
        // Dado um tipo ausente ou repetido na lista oferecida pelo SEI,
        // Quando executo documento criar, Entao a recusa explica a causa.
        use crate::documento_criar::ErroCriar;
        assert_eq!(
            mensagem_da_recusa(&ErroPreviaCriacao::Tipo(ErroCriar::TipoAusente)),
            "tipo de documento não oferecido pelo SEI"
        );
        assert_eq!(
            mensagem_da_recusa(&ErroPreviaCriacao::Tipo(ErroCriar::TipoAmbiguo)),
            "mais de um tipo de documento com o nome solicitado"
        );
        assert_eq!(
            mensagem_da_recusa(&ErroPreviaCriacao::ProcessoNaoAbertoNaUnidade),
            "processo não está aberto na unidade da sessão"
        );
    }

    #[test]
    fn recusa_de_nivel_antes_da_sessao_explica_acao_necessaria() {
        assert_eq!(
            mensagem_da_recusa(&ErroPreviaCriacao::NivelNaoInformado),
            "informe --nivel publico para confirmar a criação"
        );
        assert_eq!(
            mensagem_da_recusa(&ErroPreviaCriacao::NivelRestritoSemHipotese),
            "nível não público exige hipótese legal não disponível nesta operação"
        );
    }

    #[test]
    fn recusa_de_cadastro_ou_arvore_informa_o_impedimento_observado() {
        assert_eq!(
            mensagem_da_recusa(&ErroPreviaCriacao::SubmissaoNaoComprovada),
            "o SEI não apresentou uma forma reconhecida de concluir a criação"
        );
        assert_eq!(
            mensagem_da_recusa(&ErroPreviaCriacao::ArvoreIncompleta),
            "árvore do processo contém pastas não carregadas; criação não pode ser comprovada"
        );
        assert_eq!(
            mensagem_da_recusa(&ErroPreviaCriacao::FotoInicialInconsistente),
            "árvore inicial não permite distinguir os documentos com segurança"
        );
    }
}
