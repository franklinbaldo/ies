//! Regressões adversariais de escrita da atribuição: recusas, concorrência e prova.
//! Cenários sintéticos compartilham o roteiro e os orçamentos da fatia #264.
use super::*;

/// Caso de concorrência: outro agente publicou o resultado desejado durante
/// o POST recusado. A leitura coincidente NÃO prova que nosso POST escreveu.
#[test]
fn http_403_e_atribuicao_concorrente_nao_viram_sucesso() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(Ok((
        403,
        "<html><body>Acesso negado.</body></html>".to_owned(),
        None,
    )));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(
        erro,
        ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            falha_do_envio: Some(Box::new(
                crate::formulario_web::ErroSubmissaoFormulario::Transporte(
                    FalhaTransporte::Status(crate::transporte::StatusHttpRecusado {
                        status: 403,
                        mensagem_do_servidor: Some("Acesso negado.".into())
                    })
                )
            )),
            leitura: None,
            atribuicao_relida: Some("aluno2".into()),
            requisicoes: 6
        }
    );
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6); // 4 GET prévia + POST único + GET independente
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

/// Uma validação da Infra é recusa expressa, mesmo que outra sessão tenha
/// produzido a atribuição desejada antes da releitura da árvore.
#[test]
fn validacao_da_infra_e_alteracao_concorrente_nao_viram_sucesso() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(ok(
        "<html><body><textarea id=\"txaInfraValidacao\">Atribuição recusada.</textarea></body></html>".to_owned(),
    ));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(
        erro,
        ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            falha_do_envio: Some(Box::new(
                crate::formulario_web::ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(
                    "Atribuição recusada.".into()
                )
            )),
            leitura: None,
            atribuicao_relida: Some("aluno2".into()),
            requisicoes: 6
        }
    );
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

/// 302 é dado, nunca navegação automática; destino de ação diferente não
/// pode ser aceito como resposta comprovadora, ainda que a árvore coincida.
#[test]
fn redirecionamento_para_acao_estranha_nao_comprova_atribuicao() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302(
        "controlador.php?acao=procedimento_excluir&id_procedimento=999",
    ));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

/// Mesmo controlador e ação esperada não bastam se o 302 publica outro
/// processo. A prova da árvore coincidente pode ser ato de outro agente.
#[test]
fn redirecionamento_para_processo_divergente_nao_comprova_atribuicao() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302(
        "controlador.php?acao=arvore_visualizar&id_procedimento=999",
    ));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

/// Regressão adversarial: um 302 genérico para a árvore não vincula o
/// resultado do POST ao processo. Outra sessão pode publicar a atribuição
/// desejada antes do GET de prova; isso continua efeito incerto.
#[test]
fn arvore_sem_id_e_atribuicao_concorrente_nao_comprovam_nosso_post() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302("controlador.php?acao=arvore_visualizar"));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6); // 4 GET + POST único + GET independente
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

/// Na remoção, uma árvore sem identidade também não demonstra que o
/// POST removeu: outra sessão pode ter removido antes da releitura.
#[test]
fn arvore_sem_id_e_remocao_concorrente_nao_comprovam_nosso_post() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302("controlador.php?acao=arvore_visualizar"));
    respostas.push(ok(arvore_atual("")));
    let (resultado, pedidos) = rodar_confirmacao(None, respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("null"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

// Observação pré-RED: o 302 de aceite do Todos @323eb01 publica
// acao_origem=procedimento_atribuicao_cadastrar. Sem essa vinculação,
// a árvore coincidente pode refletir escrita concorrente de outro agente.
#[test]
fn arvore_com_id_mas_sem_origem_nao_comprova_atribuicao_concorrente() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302(&format!(
        "controlador.php?acao=arvore_visualizar&id_procedimento={ID}"
    )));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6); // 4 GET de prévia + POST + GET de prova
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn arvore_com_id_mas_sem_origem_nao_comprova_remocao_concorrente() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302(&format!(
        "controlador.php?acao=arvore_visualizar&id_procedimento={ID}"
    )));
    respostas.push(ok(arvore_atual("")));
    let (resultado, pedidos) = rodar_confirmacao(None, respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("null"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

/// Regressão de segurança: voltar à tela principal não identifica o processo
/// modificado. Uma atribuição concorrente pode fazer a releitura coincidir.
/// Caso seguro derivado da política de falhar fechado para Location genérica.
#[test]
fn principal_sem_id_nao_comprova_atribuicao_concorrente() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302("controlador.php?acao=principal"));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6); // 4 GET + 1 POST + 1 GET independente
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn principal_com_id_explicito_confirma_apos_releitura() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302(&format!(
        "controlador.php?acao=principal&id_procedimento={ID}&acao_origem=procedimento_atribuicao_cadastrar"
    )));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let feito = resultado.unwrap();
    assert_eq!(feito.estado, EstadoDaAtribuicao::AlteracaoConfirmada);
    assert_eq!(feito.requisicoes, 6);
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}
/// Regressão da fronteira entre POST e releitura: se a escrita já foi
/// tentada, o erro HTTP da leitura de prova não pode desaparecer do
/// diagnóstico público de efeito incerto.
#[test]
fn http_503_da_releitura_pos_post_preserva_status_e_mensagem_sem_retry() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(retorno_arvore_com_identidade_observado());
    respostas.push(Ok((
        503,
        "<html><body><div id=\"divInfraExcecao\">Serviço indisponível.</div></body></html>"
            .to_owned(),
        None,
    )));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(
        erro,
        ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            falha_do_envio: None,
            leitura: Some(Box::new(
                crate::processo_obter::ErroProcessoObter::Transporte(FalhaTransporte::Status(
                    crate::transporte::StatusHttpRecusado {
                        status: 503,
                        mensagem_do_servidor: Some("Serviço indisponível.".into())
                    }
                ))
            )),
            atribuicao_relida: None,
            requisicoes: 6
        }
    );
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6); // 4 GET + 1 POST + 1 GET de prova
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

/// Falha de rede do POST não encobre a recusa HTTP da nova leitura:
/// mesmo sem resposta do envio, nunca repetir o POST.
#[test]
fn post_sem_resposta_e_releitura_503_permanecem_nao_confirmado() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(Err(FalhaTransporte::Rede));
    respostas.push(Ok((
        503,
        "<html><body><div id=\"divInfraExcecao\">Serviço indisponível.</div></body></html>"
            .to_owned(),
        None,
    )));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(
        erro,
        ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            falha_do_envio: Some(Box::new(
                crate::formulario_web::ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede)
            )),
            leitura: Some(Box::new(
                crate::processo_obter::ErroProcessoObter::Transporte(FalhaTransporte::Status(
                    crate::transporte::StatusHttpRecusado {
                        status: 503,
                        mensagem_do_servidor: Some("Serviço indisponível.".into())
                    }
                ))
            )),
            atribuicao_relida: None,
            requisicoes: 6
        }
    );
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

/// O diagnóstico público de uma escrita incerta deve contar também o
/// POST tentado e o GET de prova, sem inventar sucesso por coincidência.
#[test]
fn falha_concorrente_publica_orcamento_exato_em_json() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(Ok((
        403,
        "<html><body>Acesso negado.</body></html>".to_owned(),
        None,
    )));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(
        erro,
        ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            falha_do_envio: Some(Box::new(
                crate::formulario_web::ErroSubmissaoFormulario::Transporte(
                    FalhaTransporte::Status(crate::transporte::StatusHttpRecusado {
                        status: 403,
                        mensagem_do_servidor: Some("Acesso negado.".into())
                    })
                )
            )),
            leitura: None,
            atribuicao_relida: Some("aluno2".into()),
            requisicoes: 6
        }
    );
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01:
// o aceite observado é 302 para arvore_visualizar. Um HTML 200 genérico,
// sem marcador positivo do POST, NÃO prova quem mudou a atribuição.
#[test]
fn html_200_generico_e_atribuicao_concorrente_nao_comprovam_post() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(ok("<html><body>Operação recebida.</body></html>".to_owned()));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6); // 4 GET + 1 POST + 1 GET de prova
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

// O mesmo vale para remover: ausência relida pode ter sido provocada
// por outro agente, não pelo POST que recebeu HTML 200 genérico.
#[test]
fn html_200_generico_e_remocao_concorrente_nao_comprovam_post() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(ok("<html><body>Operação recebida.</body></html>".to_owned()));
    respostas.push(ok(arvore_atual("")));
    let (resultado, pedidos) = rodar_confirmacao(None, respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("null"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}
// Caso adversarial: outro agente atribui o mesmo usuário enquanto nosso POST
// fica sem resposta. Releitura coincidente é estado observado, não autoria.
#[test]
fn sem_resposta_e_atribuicao_concorrente_exigem_nao_confirmado() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(Err(FalhaTransporte::Rede));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(
        erro,
        ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            falha_do_envio: Some(Box::new(
                crate::formulario_web::ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede)
            )),
            leitura: None,
            atribuicao_relida: Some("aluno2".into()),
            requisicoes: 6
        }
    );
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6); // 4 GET + 1 POST + 1 GET
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

// O mesmo risco existe na remoção: a ausência de atribuição na árvore
// pode ter sido causada por outra sessão após nosso POST sem resposta.
#[test]
fn sem_resposta_e_remocao_concorrente_exigem_nao_confirmado() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(Err(FalhaTransporte::Rede));
    respostas.push(ok(arvore_atual("")));
    let (resultado, pedidos) = rodar_confirmacao(None, respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(
        erro,
        ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            falha_do_envio: Some(Box::new(
                crate::formulario_web::ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede)
            )),
            leitura: None,
            atribuicao_relida: Some(String::new()),
            requisicoes: 6
        }
    );
    assert_eq!(pedidos, pedidos_confirmacao("null"));
    assert_eq!(pedidos.len(), 6); // 4 GET + 1 POST + 1 GET
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}
