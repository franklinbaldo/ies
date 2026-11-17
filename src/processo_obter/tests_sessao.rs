//! #222: as requisições de `ies processo NUP` usam o cookie do retrato
//! coerente da sessão, nunca o arquivo relido no meio do comando. Um
//! relogin concorrente (#200) grava outra geração depois da leitura: o
//! roteiro confere, em cada pedido, que o cookie é o lido (`COOKIE`).
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::transporte::sessao::CookieDaSessao;

#[test]
fn relogin_concorrente_nao_muda_o_cookie_das_requisicoes_seguintes() {
    let amb = ambiente();
    let lido = CookieDaSessao::ler(&amb.cookie);
    // O outro processo relogou: o arquivo agora é de outra geração.
    std::fs::write(&amb.cookie, "PHPSESSID=OUTRA_GERACAO\n").unwrap();
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &lido,
    };
    let nup = NumeroProcesso::novo(crate::arvore_processo::tests::NUP).unwrap();
    let pedido = PedidoDoProcesso {
        nup: &nup,
        unidade: None,
        andamentos: PedidoDeAndamentos::Omitidos,
        propriedades: false,
        marcadores: false,
        unidade_exigida: false,
        leitura_das_pastas: crate::processo_obter::LeituraDasPastas::Estrita,
    };
    let mut roteiro = Roteiro::novo(vec![
        ok(controle(&[(ID, crate::arvore_processo::tests::NUP)])),
        ok(trabalhar(ID)),
        ok(arvore_de(ID, crate::arvore_processo::tests::NUP)),
    ]);
    let aberto = obter_processo(&sessao, &pedido, &amb.indice, &mut roteiro).unwrap();
    assert_eq!(aberto.requisicoes, 3, "nenhuma requisição a mais");
    assert_eq!(roteiro.pedidos.len(), 3);
    // E o índice aprendeu sob a geração do cookie lido, não da gravada.
    let (inst, g) = (instalacao(), geracao());
    assert!(
        amb.indice
            .link(&inst, chave(&g, AcaoDoSei::ProcedimentoTrabalhar, ID))
            .is_some()
    );
    let outra = crate::sessao_login::geracao_da_sessao("PHPSESSID=OUTRA_GERACAO");
    assert!(
        amb.indice
            .link(&inst, chave(&outra, AcaoDoSei::ProcedimentoTrabalhar, ID))
            .is_none()
    );
}
