//! Regressão adversarial da concorrência entre agentes na conta SEI.
use super::tests::{cadastro, lista, pedido, tela_controle};
use super::tests_candidatos::{
    arvore_com_propriedades, gerados_com_candidato, propriedades_correspondentes,
};
use super::*;
use crate::processo_concluir::tests_apoio::{NUP, ambiente, ok, rodar_com, trabalhar};
use crate::transporte::FalhaTransporte;

/// A conta de treinamento e compartilhada entre agentes. Mesmo com um unico
/// processo novo com dados identicos, a fotografia de Gerados nao revela
/// qual POST criou esse processo quando a escrita nao devolveu resposta.
#[test]
fn red_candidato_identico_de_outro_agente_nao_prova_autoria_do_post() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_candidato()), // poderia ter sido criado por outro agente
        ok(trabalhar()),
        ok(arvore_com_propriedades()),
        ok(propriedades_correspondentes()),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    let erro = resultado.expect_err("sem resposta ao POST, candidato unico nao prova causalidade");
    assert_eq!(erro.saida(), 3);
    assert_eq!(erro.subcodigo(), "candidato_unico_sem_prova_causal");
    let diagnostico = erro
        .diagnostico_apos_post()
        .expect("preservar provas observadas");
    assert_eq!(diagnostico.causa_post, "rede");
    assert_eq!(diagnostico.causa_releitura, None);
    assert_eq!(
        diagnostico.candidatos_compativeis,
        Some(vec![NUP.to_owned()])
    );
    assert_eq!(requests.len(), 8); // 3 GET + 1 POST + 1 GET Gerados + 3 GET prova
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

#[test]
fn gerados_com_mesmo_nup_em_dois_ids_recusa_antes_do_post_criador() {
    // A tela publica duas linhas e cobertura integral, mas ambas referem
    // o mesmo NUP sob IDs diferentes. Não é fotografia segura para escrita.
    let amb = ambiente();
    let controle = super::tests_candidatos::gerados_com_dois_candidatos()
        .replace(super::tests_candidatos::SEGUNDO_NUP, NUP);
    let respostas = vec![ok(controle), ok(lista(false)), ok(cadastro())];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    assert!(matches!(
        resultado,
        Err(ErroCriar::Fotografia(
            reconciliacao::ErroFotografia::NupRepetido
        ))
    ));
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|r| r.metodo == "GET"));
}
