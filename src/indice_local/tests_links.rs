//! Links assinados: vida da geração de sessão, gravação literal, recusas
//! da URL e redação. Hashes sintéticos.
use super::tests::{instalacao, pasta};
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::instalacao::Instalacao;

const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "0123456789abcdef0123456789ABCDEF0123456789abcdef0123456789abcdef";
const G1: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const G2: &str = "2222222222222222222222222222222222222222222222222222222222222222";

fn chave(geracao: &str) -> ChaveDoLink<'_> {
    ChaveDoLink {
        geracao_sessao: geracao,
        unidade: "110",
        acao: AcaoDoSei::ProcedimentoTrabalhar,
        alvo: "7",
    }
}

fn url(hash: &str) -> String {
    format!(
        "https://sei.exemplo/sei/controlador.php?acao=procedimento_trabalhar\
         &acao_origem=procedimento_controlar&id_procedimento=7\
         &infra_sistema=100000100&infra_unidade_atual=110&infra_hash={hash}"
    )
}

fn indice(nome: &str) -> IndiceLocal {
    IndiceLocal::em(pasta(nome).join(NOME_DO_ARQUIVO))
}

#[test]
fn link_volta_literal_so_na_mesma_geracao() {
    let indice = indice("link-geracao");
    let sei = instalacao();
    indice
        .registrar_link(&sei, chave(G1), &url(HASH_B))
        .unwrap();
    let guardado = indice.link(&sei, chave(G1)).unwrap();
    assert_eq!(guardado.url(), url(HASH_B));
    assert_eq!(indice.link(&sei, chave(G2)), None);
    let outra_unidade = ChaveDoLink {
        unidade: "111",
        ..chave(G1)
    };
    assert_eq!(indice.link(&sei, outra_unidade), None);
    let outro_objeto = ChaveDoLink {
        alvo: "8",
        ..chave(G1)
    };
    assert_eq!(indice.link(&sei, outro_objeto), None);
    let outra = Instalacao::interpretar("https://sei.exemplo/outro").unwrap();
    assert_eq!(indice.link(&outra, chave(G1)), None);
    let outro_host = Instalacao::interpretar("https://outro.exemplo/sei").unwrap();
    assert_eq!(indice.link(&outro_host, chave(G1)), None);
}

#[test]
fn nova_observacao_da_mesma_chave_substitui() {
    let indice = indice("link-substitui");
    let sei = instalacao();
    indice
        .registrar_link(&sei, chave(G1), &url(HASH_A))
        .unwrap();
    indice
        .registrar_link(&sei, chave(G1), &url(HASH_B))
        .unwrap();
    assert_eq!(indice.link(&sei, chave(G1)).unwrap().url(), url(HASH_B));
}

#[test]
fn descartar_outras_geracoes_mantem_a_atual_e_outras_instalacoes() {
    let indice = indice("link-descartar");
    let sei = instalacao();
    let outra = Instalacao::interpretar("https://sei.exemplo/outro").unwrap();
    let url_outra = url(HASH_A).replace("/sei/", "/outro/");
    indice
        .registrar_link(&sei, chave(G1), &url(HASH_A))
        .unwrap();
    indice
        .registrar_link(&sei, chave(G2), &url(HASH_B))
        .unwrap();
    indice
        .registrar_link(&outra, chave(G1), &url_outra)
        .unwrap();
    indice.descartar_outras_geracoes(&sei, G2);
    assert_eq!(indice.link(&sei, chave(G1)), None);
    assert_eq!(indice.link(&sei, chave(G2)).unwrap().url(), url(HASH_B));
    assert_eq!(indice.link(&outra, chave(G1)).unwrap().url(), url_outra);
}

#[test]
fn esquecer_link_apaga_so_a_chave() {
    let indice = indice("link-esquecer");
    let sei = instalacao();
    let outra_acao = ChaveDoLink {
        acao: AcaoDoSei::ArvoreVisualizar,
        ..chave(G1)
    };
    let url_arvore = url(HASH_A).replace("procedimento_trabalhar", "arvore_visualizar");
    indice
        .registrar_link(&sei, chave(G1), &url(HASH_B))
        .unwrap();
    indice
        .registrar_link(&sei, outra_acao, &url_arvore)
        .unwrap();
    indice.esquecer_link(&sei, chave(G1));
    assert_eq!(indice.link(&sei, chave(G1)), None);
    assert_eq!(indice.link(&sei, outra_acao).unwrap().url(), url_arvore);
}

/// Cada URL recusada e o fato verificável que a recusa.
fn urls_recusadas(boa: &str) -> Vec<(String, RegistroRecusado)> {
    vec![
        (
            boa.replace("https://", "http://"),
            RegistroRecusado::LinkNaoHttps,
        ),
        (
            boa.replace("https://", "HTTPS://"),
            RegistroRecusado::LinkNaoHttps,
        ),
        (
            boa.replace("controlador.php?", "controlador.php?a=b c&"),
            RegistroRecusado::LinkComCaractereProibido,
        ),
        (
            format!("{boa}#ancora"),
            RegistroRecusado::LinkComCaractereProibido,
        ),
        (
            boa.replace("sei/controlador", "sei\\controlador"),
            RegistroRecusado::LinkComCaractereProibido,
        ),
        (
            boa.replace("sei.exemplo", "outro.exemplo"),
            RegistroRecusado::LinkForaDaInstalacao,
        ),
        (
            boa.replace("/sei/", "/outro/"),
            RegistroRecusado::LinkForaDaInstalacao,
        ),
        (
            boa.replace("controlador.php", "controlador_externo.php"),
            RegistroRecusado::LinkForaDaInstalacao,
        ),
        (
            boa.replace("https://sei.exemplo", "https://u@sei.exemplo"),
            RegistroRecusado::LinkForaDaInstalacao,
        ),
        (
            boa.replace("controlador.php?", "controlador.php"),
            RegistroRecusado::LinkForaDaInstalacao,
        ),
        (
            boa.replace("acao_origem", "Acao_origem"),
            RegistroRecusado::LinkComConsultaNaoCanonica,
        ),
        (
            boa.replace("infra_sistema=100000100", "infra_sistema=1;2"),
            RegistroRecusado::LinkComConsultaNaoCanonica,
        ),
        (
            boa.replace("&id_procedimento=7", "&id_procedimento"),
            RegistroRecusado::LinkComConsultaNaoCanonica,
        ),
        (
            boa.replace("&id_procedimento=7", "&&id_procedimento=7"),
            RegistroRecusado::LinkComConsultaNaoCanonica,
        ),
        (
            boa.replace("acao=procedimento_trabalhar", "acao=arvore_visualizar"),
            RegistroRecusado::LinkSemAcaoUnicaIgual,
        ),
        (
            format!("{boa}&acao=procedimento_trabalhar"),
            RegistroRecusado::LinkSemAcaoUnicaIgual,
        ),
        (
            boa.replace("&infra_hash", "&outro"),
            RegistroRecusado::LinkSemInfraHashUnico,
        ),
        (
            format!("{boa}&infra_hash={HASH_B}"),
            RegistroRecusado::LinkSemInfraHashUnico,
        ),
        (
            boa.replace(HASH_A, &HASH_A[1..]),
            RegistroRecusado::LinkSemInfraHashUnico,
        ),
        (
            boa.replace(HASH_A, &HASH_A.replace('a', "g")),
            RegistroRecusado::LinkSemInfraHashUnico,
        ),
        (
            format!("{boa}&acao_ajax=x"),
            RegistroRecusado::LinkComAcaoAjax,
        ),
        (
            boa.replace("infra_unidade_atual=110", "infra_unidade_atual=111"),
            RegistroRecusado::LinkDeOutraUnidade,
        ),
    ]
}

#[test]
fn urls_recusadas_por_fato_verificavel() {
    let indice = indice("link-recusas");
    let sei = instalacao();
    let casos = urls_recusadas(&url(HASH_A));
    for (texto, esperado) in casos {
        assert_eq!(
            indice.registrar_link(&sei, chave(G1), &texto),
            Err(esperado),
            "{esperado:?}"
        );
    }
    assert_eq!(indice.link(&sei, chave(G1)), None);
    assert!(indice.ligado());
}

#[test]
fn chaves_invalidas_recusadas() {
    let indice = indice("link-chaves");
    let sei = instalacao();
    let casos = [
        (
            ChaveDoLink {
                geracao_sessao: "",
                ..chave(G1)
            },
            RegistroRecusado::GeracaoInvalida,
        ),
        (
            ChaveDoLink {
                geracao_sessao: "a b",
                ..chave(G1)
            },
            RegistroRecusado::GeracaoInvalida,
        ),
        (
            ChaveDoLink {
                unidade: "",
                ..chave(G1)
            },
            RegistroRecusado::UnidadeInvalida,
        ),
        (
            ChaveDoLink {
                unidade: "1\n",
                ..chave(G1)
            },
            RegistroRecusado::UnidadeInvalida,
        ),
        (
            ChaveDoLink {
                alvo: "a\tb",
                ..chave(G1)
            },
            RegistroRecusado::ObjetoInvalido,
        ),
    ];
    for (chave, esperado) in casos {
        assert_eq!(
            indice.registrar_link(&sei, chave, &url(HASH_A)),
            Err(esperado),
            "{esperado:?}"
        );
    }
}

#[test]
fn objeto_vazio_e_sem_unidade_na_url_sao_aceitos() {
    let indice = indice("link-sem-objeto");
    let sei = instalacao();
    let pesquisa = ChaveDoLink {
        acao: AcaoDoSei::ProtocoloPesquisar,
        alvo: "",
        ..chave(G1)
    };
    let texto = format!(
        "https://sei.exemplo/sei/controlador.php?acao=protocolo_pesquisar&infra_hash={HASH_A}"
    );
    indice.registrar_link(&sei, pesquisa, &texto).unwrap();
    assert_eq!(indice.link(&sei, pesquisa).unwrap().url(), texto);
}

#[test]
fn debug_redige_url_e_hash() {
    let indice = indice("link-debug");
    let sei = instalacao();
    indice
        .registrar_link(&sei, chave(G1), &url(HASH_B))
        .unwrap();
    let guardado = indice.link(&sei, chave(G1));
    let texto = format!("{guardado:?} {indice:?}");
    for segredo in [HASH_B, "sei.exemplo", "infra_hash", G1] {
        assert!(!texto.contains(segredo), "{texto}");
    }
    assert!(texto.contains("redigida"), "{texto}");
}

#[test]
fn troca_de_unidade_apaga_todos_os_links_da_instalacao() {
    let indice = indice("link-troca");
    let sei = instalacao();
    let outra = Instalacao::interpretar("https://sei.exemplo/outro").unwrap();
    let url_outra = url(HASH_A).replace("/sei/", "/outro/");
    indice
        .registrar_link(&sei, chave(G1), &url(HASH_A))
        .unwrap();
    indice
        .registrar_link(&sei, chave(G2), &url(HASH_B))
        .unwrap();
    indice
        .registrar_link(&outra, chave(G1), &url_outra)
        .unwrap();
    assert_eq!(indice.esquecer_links_da_instalacao(&sei), Some(2));
    assert_eq!(indice.link(&sei, chave(G1)), None);
    assert_eq!(indice.link(&sei, chave(G2)), None);
    assert!(indice.link(&outra, chave(G1)).is_some());
    assert_eq!(indice.esquecer_links_da_instalacao(&sei), Some(0));
    assert_eq!(
        IndiceLocal::desligado().esquecer_links_da_instalacao(&sei),
        None
    );
}
