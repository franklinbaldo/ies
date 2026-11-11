//! Testes técnicos de apresentação dos estados internos de escrita.
//! A escrita CLI pública permanece fechada, sem fazer POST externo.
use super::*;

#[test]
fn confirmacao_injetavel_exibe_sucesso_sem_chamar_de_previa() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(retorno_arvore_com_identidade_observado());
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let feito = resultado.unwrap();
    let texto = crate::processo_atribuir_cli::texto_humano(&feito);
    assert!(
        texto.starts_with("Atribuição confirmada: processo "),
        "{texto}"
    );
    assert!(texto.contains("Atribuição anterior: aluno20."), "{texto}");
    assert!(texto.contains("Atribuição agora: aluno2."), "{texto}");
    assert!(texto.contains("Requisições: 6."), "{texto}");
    assert!(!texto.contains("Prévia:"), "{texto}");
    let json = crate::processo_atribuir_cli::json_da_previa(&feito);
    assert_eq!(json["confirmado"], true);
    assert_eq!(json["alterado"], true);
    assert_eq!(json["remover"], false);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn remocao_injetavel_distingue_ausencia_verificada_de_previa() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(retorno_arvore_com_identidade_observado());
    respostas.push(ok(arvore_atual("")));
    let (resultado, pedidos) = rodar_confirmacao(None, respostas);
    let feito = resultado.unwrap();
    let texto = crate::processo_atribuir_cli::texto_humano(&feito);
    assert!(
        texto.starts_with("Remoção confirmada: processo "),
        "{texto}"
    );
    assert!(texto.contains("Atribuição anterior: aluno20."), "{texto}");
    assert!(texto.contains("Atribuição agora: nenhuma."), "{texto}");
    assert!(texto.contains("Requisições: 6."), "{texto}");
    assert!(!texto.contains("Prévia:"), "{texto}");
    let json = crate::processo_atribuir_cli::json_da_previa(&feito);
    assert_eq!(json["confirmado"], true);
    assert_eq!(json["alterado"], true);
    assert_eq!(json["remover"], true);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn previa_nao_promete_escrita_nem_no_texto_nem_no_json() {
    let (resultado, pedidos) = rodar(Some("aluno2"), respostas_previa("aluno20"));
    let feito = resultado.unwrap();
    let texto = crate::processo_atribuir_cli::texto_humano(&feito);
    assert!(texto.starts_with("Prévia: processo "), "{texto}");
    assert!(texto.contains("Atribuição atual: aluno20."), "{texto}");
    assert!(
        texto.contains("Atribuir a aluno2; opção verificada na tela."),
        "{texto}"
    );
    assert!(!texto.contains("confirmada"), "{texto}");
    let json = crate::processo_atribuir_cli::json_da_previa(&feito);
    assert_eq!(json["confirmado"], false);
    assert_eq!(json["alterado"], false);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 0);
}
// O 302 histórico das cassettes publica acao_origem, mas não uma
// identidade decimal singular. A árvore pode coincidir após a escrita;
// a CLI não deve chamar essa coincidência de confirmação do nosso POST.
#[test]
fn redirect_historico_sem_id_expoe_incerteza_em_vez_de_confirmacao() {
    for usuario in [Some("aluno2"), None] {
        let mut respostas = respostas_previa("aluno20");
        respostas.push(r302(
            "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_atribuicao_cadastrar",
        ));
        respostas.push(ok(arvore_atual(usuario.unwrap_or(""))));
        let (resultado, pedidos) = rodar_confirmacao(usuario, respostas);
        let erro = resultado.unwrap_err();
        assert_eq!(
            erro,
            ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
                falha_do_envio: None,
                leitura: None,
                atribuicao_relida: Ok(usuario.unwrap_or("").to_owned()),
                requisicoes: 6
            }
        );
        assert_eq!(pedidos.len(), 6);
        assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
    }
}
