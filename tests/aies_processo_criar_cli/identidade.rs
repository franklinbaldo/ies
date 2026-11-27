//! #245: divergência sintética do tipo após o POST efetivo.
use super::*;

fn banco_com_tipo_substituido() -> (Connection, Instalacao) {
    let (banco, instalacao) = banco();
    // A lista inicial oferece apenas 12. A hipótese de mutação ocorre
    // no SQLite, depois de a CLI escolher o tipo, sem falsificar HTML.
    banco
        .execute_batch(
            "CREATE TEMP TRIGGER trocar_tipo_homonimo AFTER INSERT ON processo_criado
             WHEN NEW.id_tipo = '12'
             BEGIN
                 INSERT INTO tipo_processo_simulado
                     (origem, cenario, id_tipo, nome, id_assunto, assunto)
                 SELECT origem, cenario, '13', nome, id_assunto, assunto
                 FROM tipo_processo_simulado
                 WHERE origem = NEW.origem AND cenario = NEW.cenario
                   AND id_tipo = NEW.id_tipo;
                 UPDATE processo_criado SET id_tipo = '13' WHERE id = NEW.id;
             END;",
        )
        .unwrap();
    (banco, instalacao)
}

fn observar_tipo(banco: &Connection, _: &Instalacao) -> (i64, Option<String>, Option<String>) {
    banco
        .query_row(
            "SELECT COUNT(*), MIN(c.id_tipo), MIN(t.nome)
             FROM processo_criado c
             JOIN tipo_processo_simulado t ON t.origem = c.origem
               AND t.cenario = c.cenario AND t.id_tipo = c.id_tipo",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
}

#[test]
fn tipo_homonimo_persistido_nao_confirma_nem_repete_post() {
    let (saida, _, pedidos, antes, depois) = executar_em_bancada(
        &[
            "--json",
            "processo",
            "criar",
            "--tipo",
            TIPO,
            "--especificacao",
            "Cadastro de identidade divergente",
            "--confirmar",
        ],
        false,
        false,
        banco_com_tipo_substituido,
        observar_tipo,
    );
    assert_eq!(antes, (0, None, None));
    assert_eq!(depois, (1, Some("13".into()), Some(TIPO.into())));
    assert_eq!(
        saida.status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    assert!(saida.stdout.is_empty());
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(
        erro.contains("VerificacaoAposPost(ProvaDaCriacao(PropriedadesDivergentes))"),
        "{erro}"
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.starts_with("POST procedimento_gerar "))
            .count(),
        1,
    );
    assert!(pedidos.iter().any(|p| p == "POST procedimento_gerar 302"));
    assert!(pedidos.iter().any(|p| p == "GET procedimento_alterar 200"));
}
