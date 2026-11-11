//! #786: o índice segue com o padrão, mas guarda a falha tipada que a CLI
//! publica em `avisos`; o aceite da entrada por id é um enum, não
//! `Option<bool>`. Diretórios temporários, sem rede.
use super::abertura::ErroAberturaDoIndice;
use super::tests::{instalacao, pasta};
use super::*;
use crate::arquivo_privado::FalhaDaGravacao;
use std::fs;

#[test]
fn arquivo_no_lugar_da_pasta_devolve_o_padrao_e_guarda_a_causa() {
    let raiz = pasta("falha-pasta-arquivo");
    let ocupado = raiz.join("ocupado");
    fs::write(&ocupado, b"").unwrap();
    let indice = IndiceLocal::em(ocupado.join(NOME_DO_ARQUIVO));

    assert_eq!(indice.id_por_nup(&instalacao(), "N1"), None);
    assert!(!indice.ligado());
    let avisos = indice.avisos();
    assert!(
        matches!(
            avisos.as_slice(),
            [FalhaDoIndice::Banco(
                ErroAberturaDoIndice::DiretoriosRecusados(
                    FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio
                )
            )]
        ),
        "{avisos:?}"
    );
    // A forma publicada: variante e motivo, nunca o caminho.
    let publicado = serde_json::json!(avisos);
    assert_eq!(
        publicado,
        serde_json::json!([{
            "falha": "banco",
            "causa": {
                "erro": "diretorios_recusados",
                "gravacao": "NivelDoCaminhoNaoEhDiretorio"
            }
        }])
    );
    assert!(!publicado.to_string().contains("falha-pasta-arquivo"));

    // Desligado, o índice não tenta de novo nem repete o aviso.
    indice.registrar_processo(&instalacao(), "N1", "10");
    assert!(indice.avisos().is_empty());
}

#[cfg(unix)]
#[test]
fn banco_link_simbolico_vira_aviso_arquivo_nao_privado() {
    let raiz = pasta("falha-symlink");
    let alvo = raiz.join("alvo");
    fs::write(&alvo, b"").unwrap();
    tests::definir_modo(&alvo, 0o600);
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    std::os::unix::fs::symlink(&alvo, &caminho).unwrap();
    let indice = IndiceLocal::em(caminho);

    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    let avisos = indice.avisos();
    assert!(
        matches!(
            avisos.as_slice(),
            [FalhaDoIndice::Banco(
                ErroAberturaDoIndice::ArquivoNaoPrivado
            )]
        ),
        "{avisos:?}"
    );
    assert_eq!(
        serde_json::json!(avisos),
        serde_json::json!([{"falha": "banco", "causa": {"erro": "arquivo_nao_privado"}}])
    );
}

#[test]
fn indice_desligado_por_configuracao_nao_e_falha() {
    let indice = IndiceLocal::desligado();
    indice.registrar_processo(&instalacao(), "N1", "10");
    assert_eq!(indice.id_por_nup(&instalacao(), "N1"), None);
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert!(indice.avisos().is_empty());
}

#[test]
fn avisos_sao_omitidos_da_saida_sem_falha_e_entram_com_falha() {
    let indice = IndiceLocal::em(pasta("falha-saida").join(NOME_DO_ARQUIVO));
    let mut valor = serde_json::json!({"origem": "web"});
    anexar_avisos(&mut valor, &indice);
    assert_eq!(valor, serde_json::json!({"origem": "web"}));

    indice.guardar_link(&instalacao(), chave_sem_unidade(), "https://x");
    let mut lista = serde_json::json!([{"origem": "web"}]);
    anexar_avisos(&mut lista, &indice);
    assert_eq!(
        lista,
        serde_json::json!([{
            "origem": "web",
            "avisos": [{"falha": "registro_recusado", "causa": "unidade_invalida"}]
        }])
    );
    // Os avisos saem uma vez: a segunda saída não os repete.
    let mut de_novo = serde_json::json!({"origem": "web"});
    anexar_avisos(&mut de_novo, &indice);
    assert_eq!(de_novo, serde_json::json!({"origem": "web"}));
}

fn chave_sem_unidade() -> ChaveDoLink<'static> {
    ChaveDoLink {
        geracao_sessao: "g",
        unidade: "",
        acao: crate::acao_do_sei::AcaoDoSei::ProtocoloPesquisar,
        alvo: "",
    }
}

#[test]
fn aceite_da_entrada_por_id_aceita_recusada_e_sem_registro() {
    let indice = IndiceLocal::em(pasta("falha-aceite").join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    assert_eq!(indice.entrada_por_id(&sei), None);
    assert_eq!(indice.entrada_por_id_com_documento(&sei), None);
    indice.registrar_entrada_por_id(&sei, true);
    assert_eq!(indice.entrada_por_id(&sei), Some(AceiteDaEntrada::Aceita));
    assert_eq!(indice.entrada_por_id_com_documento(&sei), None);
    indice.registrar_entrada_por_id_com_documento(&sei, false);
    assert_eq!(
        indice.entrada_por_id_com_documento(&sei),
        Some(AceiteDaEntrada::Recusada)
    );
    assert!(indice.avisos().is_empty());
}

#[test]
fn estado_do_aceite_fora_do_conjunto_vira_aviso_tipado() {
    let caminho = pasta("falha-aceite-fora").join(NOME_DO_ARQUIVO);
    let indice = IndiceLocal::em(caminho.clone());
    indice.registrar_entrada_por_id(&instalacao(), true);
    // Só um banco adulterado por fora do IES passa pelo CHECK (0, 1).
    let banco = rusqlite::Connection::open(&caminho).unwrap();
    banco
        .execute_batch(
            "PRAGMA ignore_check_constraints = ON;
             UPDATE instalacao SET entrada_por_id = 7;",
        )
        .unwrap();
    drop(banco);

    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    let avisos = indice.avisos();
    assert!(
        matches!(
            avisos.as_slice(),
            [FalhaDoIndice::EstadoDoAceiteForaDoConjunto(7)]
        ),
        "{avisos:?}"
    );
    assert_eq!(
        serde_json::json!(avisos),
        serde_json::json!([{"falha": "estado_do_aceite_fora_do_conjunto", "causa": 7}])
    );
    assert!(indice.ligado());
}

#[test]
fn identidade_invalida_recusada_sem_gravar() {
    let raiz = pasta("identidade-invalida");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    for id in ["", "12a", "1&acao=x", "-1", "123456789012345678901"] {
        indice.registrar_processo(&sei, "N", id);
        let avisos = indice.avisos();
        assert!(
            matches!(
                avisos.as_slice(),
                [FalhaDoIndice::RegistroRecusado(
                    RegistroRecusado::IdProcedimentoInvalido
                )]
            ),
            "{id}: {avisos:?}"
        );
    }
    for nup in ["", "a\nb", "a\u{0}b"] {
        indice.registrar_processo(&sei, nup, "1");
        let avisos = indice.avisos();
        assert!(
            matches!(
                avisos.as_slice(),
                [FalhaDoIndice::RegistroRecusado(
                    RegistroRecusado::NupInvalido
                )]
            ),
            "{avisos:?}"
        );
    }
    assert_eq!(indice.id_por_nup(&sei, "N"), None);
    assert!(indice.ligado());
}
