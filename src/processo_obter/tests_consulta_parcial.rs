//! BDD da leitura parcial segura de metadados quando pasta está incompleta (Refs #342).
//! BDD-342-01: metadados comprovados, documentos_incluidos: false, documentos: [] e aviso.
//! BDD-342-02: falha fechada quando identidade do processo não puder ser comprovada.
use super::consulta_parcial::obter_processo_informativo;
use super::tests_apoio::{
    Ambiente, ORIGEM, Resposta, Roteiro, ambiente, controle, ok, trabalhar, url_controle,
};
use super::*;
use crate::arvore_processo::tests::NUP;
use crate::arvore_processo::tests_pastas::{
    ID, No, acoes_das_pastas, arvore_em_pastas, declaracoes, url_abrir,
};
use crate::arvore_processo::{ErroArvore, ler_arvore, ler_arvore_parcial};
use crate::processo_obter_cli::json_do_processo;
use serde_json::json;

fn pasta_incompleta_pela_metade() -> String {
    let declaradas: &[(usize, &[&str])] = &[(1, &["900", "901"])];
    let nos = vec![
        No::Processo,
        No::Pasta(1, true),
        No::Aguarde(1),
        No::Documento("21", "PASTA1".into(), Some("900")),
    ];
    arvore_em_pastas(
        &declaracoes(declaradas),
        &nos,
        &acoes_das_pastas(&url_abrir(ID)),
    )
}

fn rodar_informativo(
    amb: &Ambiente,
    respostas: Vec<Resposta>,
) -> Result<ProcessoAberto, ErroProcessoObter> {
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &super::CookieDaSessao::ler(&amb.cookie),
    };
    let nup = NumeroProcesso::novo(NUP).unwrap();
    let pedido = PedidoDoProcesso {
        nup: &nup,
        unidade: None,
        andamentos: PedidoDeAndamentos::Omitidos,
        propriedades: false,
        marcadores: false,
        unidade_exigida: false,
        permitir_leitura_parcial: true,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let r = obter_processo_informativo(&sessao, &pedido, &amb.indice, &mut roteiro);
    assert_eq!(roteiro.respostas_restantes(), 0, "respostas não consumidas");
    r
}

fn rodar_escrita(
    amb: &Ambiente,
    respostas: Vec<Resposta>,
) -> Result<ProcessoAberto, ErroProcessoObter> {
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &super::CookieDaSessao::ler(&amb.cookie),
    };
    let nup = NumeroProcesso::novo(NUP).unwrap();
    let pedido = PedidoDoProcesso {
        nup: &nup,
        unidade: None,
        andamentos: PedidoDeAndamentos::Omitidos,
        propriedades: false,
        marcadores: false,
        unidade_exigida: false,
        permitir_leitura_parcial: false,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let r = obter_processo(&sessao, &pedido, &amb.indice, &mut roteiro);
    assert_eq!(roteiro.respostas_restantes(), 0, "respostas não consumidas");
    r
}

#[test]
fn bdd_342_01_leitura_parcial_segura_com_aviso_quando_pasta_incompleta() {
    let html = pasta_incompleta_pela_metade();

    // 1. ler_arvore estrita continua recusando (para comandos de escrita)
    let estrita = ler_arvore(&html, NUP, ID);
    assert_eq!(estrita, Err(ErroArvore::PastaCarregadaPelaMetade));

    // 2. ler_arvore_parcial recupera metadados com degradação segura
    let parcial = ler_arvore_parcial(&html, NUP, ID).expect("deve recuperar metadados");
    assert_eq!(parcial.processo.nup, NUP);
    assert_eq!(parcial.processo.id, ID);
    assert!(!parcial.documentos_incluidos);
    assert!(parcial.documentos.is_empty());
    assert!(parcial.pastas.is_empty());
    assert_eq!(
        parcial.aviso_leitura_parcial.as_deref(),
        Some(
            "leitura parcial do processo: lista de documentos incompleta (pasta_carregada_pela_metade)"
        )
    );

    // 3. Serialização JSON observável da CLI
    let nup_tipo = NumeroProcesso::novo(NUP).unwrap();
    let aberto = ProcessoAberto {
        arvore: parcial,
        unidade: Some("TESTE".into()),
        resolvido_por: ResolvidoPor::Caixa,
        andamentos: Andamentos::Omitidos,
        propriedades: Propriedades::Omitidas,
        marcadores: Marcadores::Omitidos,
        requisicoes: 3,
    };
    let valor = json_do_processo(&nup_tipo, &aberto, false);
    assert_eq!(valor["numero"], NUP);
    assert_eq!(valor["id_procedimento"], ID);
    assert_eq!(valor["documentos_incluidos"], json!(false));
    assert_eq!(valor["documentos"], json!([]));
    assert_eq!(
        valor["aviso_leitura_parcial"],
        "leitura parcial do processo: lista de documentos incompleta (pasta_carregada_pela_metade)"
    );
}

#[test]
fn bdd_342_02_falha_fechada_quando_identidade_diverge() {
    let html = pasta_incompleta_pela_metade();

    // NUP divergente falha fechado imediatamente mesmo no modo parcial
    let r_nup = ler_arvore_parcial(&html, "00000.000000/0000-00", ID);
    assert_eq!(
        r_nup,
        Err(ErroArvore::IdentidadeDoProcessoDiferenteDaPedida)
    );

    // ID divergente falha fechado imediatamente mesmo no modo parcial
    let r_id = ler_arvore_parcial(&html, NUP, "99999");
    assert_eq!(r_id, Err(ErroArvore::IdDoProcessoNaArvoreDiferenteDoAberto));
}

#[test]
fn bdd_342_integracao_informativa_vs_escrita() {
    let amb = ambiente();
    let html_incompleto = pasta_incompleta_pela_metade();

    // Cenário Informativo: obter_processo_informativo aceita metadados com degradação
    let respostas_info = vec![
        ok(controle(&[(ID, NUP)])),
        ok(trabalhar(ID)),
        ok(html_incompleto.clone()),
    ];
    let obtido = rodar_informativo(&amb, respostas_info)
        .expect("consulta informativa deve aceitar leitura parcial");
    assert!(!obtido.arvore.documentos_incluidos);
    assert!(obtido.arvore.documentos.is_empty());
    assert!(obtido.arvore.aviso_leitura_parcial.is_some());

    // Cenário de Escrita/Mutação: obter_processo estrito recusa imediatamente com erro tipado
    let respostas_escrita = vec![
        ok(controle(&[(ID, NUP)])),
        ok(trabalhar(ID)),
        ok(html_incompleto),
    ];
    let falha = rodar_escrita(&amb, respostas_escrita)
        .expect_err("escrita não pode aceitar pasta corrompida");
    assert_eq!(
        falha,
        ErroProcessoObter::Arvore(ErroArvore::PastaCarregadaPelaMetade)
    );
}

#[test]
fn bdd_342_02_pasta_com_pai_divergente_nao_pode_ser_tratada_como_lista_incompleta() {
    let html = pasta_incompleta_pela_metade().replace(
        "\"PASTA1\",\"10\",\"javascript:abrirFecharPasta",
        "\"PASTA1\",\"99\",\"javascript:abrirFecharPasta",
    );
    assert!(html.contains("\"PASTA1\",\"99\""));
    assert_eq!(
        ler_arvore_parcial(&html, NUP, ID),
        Err(ErroArvore::PastaComPaiDiferenteDoProcesso)
    );
}
