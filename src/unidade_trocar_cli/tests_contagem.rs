//! Contagem (#184) da troca de unidade com a bancada HTTP sintética.
//! O Roteiro usa `testes_http::atender`: cada tentativa é registrada
//! antes de devolver sucesso, 302, falha de rede ou recusa do POST.
use super::*;
use crate::instalacao::Instalacao;
use crate::transporte::sessao::CookieDaSessao;
use crate::unidade_trocar::tests_apoio::{
    ID_A, ID_B, ORIGEM, Pedido, Resposta, Roteiro, arquivo_de_cookie, controle, html,
    localizacao_com_aviso, localizacao_limpa, redireciona, tela, url_controle,
};

fn executar(
    sigla: &str,
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<Troca, ErroTroca>, Vec<Pedido>, u32) {
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    let cookie = arquivo_de_cookie("contagem");
    let url = url_controle(ID_A);
    let sessao = SessaoDaTroca {
        url_do_controle: &url,
        instalacao: &instalacao,
        cookie: &CookieDaSessao::ler(&cookie),
    };
    let pedido = PedidoDeTroca {
        sigla,
        confirmar,
        unidade_esperada: None,
    };
    let mut contador = Contador {
        envio: Roteiro::novo(respostas),
        pedidos: ContagemDeRequisicoes::default(),
    };
    let resultado = trocar_unidade(&sessao, &pedido, &mut contador);
    let total = u32::try_from(contador.pedidos.total()).unwrap();
    let pedidos = contador.envio.pedidos;
    std::fs::remove_dir_all(cookie.parent().unwrap()).unwrap();
    (resultado, pedidos, total)
}

fn inicio() -> Vec<Resposta> {
    vec![html(controle("PR-A", ID_A)), html(tela())]
}

#[test]
fn get_200_ja_na_unidade_conta_uma_tentativa() {
    let (resultado, pedidos, total) = executar("PR-A", false, vec![html(controle("PR-A", ID_A))]);
    assert!(resultado.is_ok());
    assert_eq!(total, 1);
    assert_eq!(pedidos.len(), 1);
    assert_eq!(pedidos[0].metodo, "GET");
}

#[test]
fn dois_302_sao_dados_e_contam_cinco_tentativas_sem_redirect_automatico() {
    let mut respostas = inicio();
    respostas.extend([
        redireciona(localizacao_com_aviso("https", ID_B)),
        redireciona(localizacao_limpa("https", ID_B)),
        html(controle("PR-B", ID_B)),
    ]);
    let (resultado, pedidos, total) = executar("PR-B", true, respostas);
    assert!(resultado.unwrap().controle_novo.is_some());
    assert_eq!(total, 5); // saída de sucesso histórica
    assert_eq!(
        pedidos.iter().map(|p| p.metodo).collect::<Vec<_>>(),
        ["GET", "GET", "POST", "GET", "GET"]
    );
}

#[test]
fn falha_de_rede_no_segundo_get_conta_as_duas_tentativas() {
    let (resultado, pedidos, total) = executar(
        "PR-B",
        false,
        vec![html(controle("PR-A", ID_A)), Err(FalhaTransporte::Rede)],
    );
    assert!(matches!(
        resultado,
        Err(ErroTroca::Transporte(FalhaTransporte::Rede))
    ));
    assert_eq!(total, 2);
    assert_eq!(
        pedidos.iter().map(|p| p.metodo).collect::<Vec<_>>(),
        ["GET", "GET"]
    );
}

#[test]
fn post_recusado_conta_tres_tentativas_inclusive_o_envio() {
    let mut respostas = inicio();
    respostas.push(Ok((403, "Acesso negado".to_owned(), None)));
    let (resultado, pedidos, total) = executar("PR-B", true, respostas);
    assert!(matches!(resultado, Err(ErroTroca::NaoConfirmado { .. })));
    assert_eq!(total, 3);
    assert_eq!(
        pedidos.iter().map(|p| p.metodo).collect::<Vec<_>>(),
        ["GET", "GET", "POST"]
    );
}
