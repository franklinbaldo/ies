//! CLI Web de controle de prazo: leitura e prévia, sem escrever no SEI.
//! A confirmação permanece recusada antes de qualquer acesso à sessão,
//! até haver POST único, reconciliação por nova leitura e testes do fluxo.
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, ler_cookie, leitura, obter_tela};
use crate::processo_obter::{SessaoDoProcesso, abrir_para_escrever};
use crate::processo_prazo::{EstadoDoPrazo, PedidoDePrazo, campos_do_pedido, interpretar_pedido, ler_tela};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, processo_concluir_cli, saida_erro};
use serde_json::{Value, json};
use std::process::ExitCode;

pub(crate) fn tentar_prazo(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Prazo { numero, ate, dias, excluir, confirmar }),
        ..
    }) = &cli.command else {
        return None;
    };
    let nup = match NumeroProcesso::novo(numero) {
        Ok(nup) => nup,
        Err(erro) => return Some(uso(cli.json, erro.subcodigo())),
    };
    let pedido = match interpretar_pedido(ate.as_deref(), *dias, *excluir) {
        Ok(pedido) => pedido,
        Err(erro) => return Some(uso(cli.json, erro.subcodigo())),
    };
    if *confirmar {
        return Some(recusar(cli.json, "escrita_de_prazo_ainda_nao_disponivel",
            "confirmação de prazo ainda não implementada com reconciliação; nenhuma requisição enviada"));
    }
    if pedido == PedidoDePrazo::Excluir {
        return Some(recusar(cli.json, "exclusao_de_prazo_ainda_nao_disponivel",
            "exclusão depende da lista de prazos da unidade; nenhuma requisição enviada"));
    }
    Some(processo_concluir_cli::com_sessao(cli, |sessao, indice| {
        previsualizar(cli, &nup, &pedido, sessao, indice)
    }))
}

fn uso(json_mode: bool, subcodigo: &str) -> ExitCode {
    if json_mode {
        println!("{}", json!({
            "codigo": "entrada_invalida",
            "subcodigo": subcodigo,
            "erro": "parâmetros de prazo inválidos",
            "retentavel": false
        }));
    } else {
        eprintln!("IES: parâmetros de prazo inválidos [{subcodigo}]");
    }
    ExitCode::from(2)
}

fn recusar(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    saida_erro::erro_fonte_http(json_mode, subcodigo, mensagem, None)
}

fn previsualizar(
    cli: &Cli,
    nup: &NumeroProcesso,
    pedido: &PedidoDePrazo,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
) -> ExitCode {
    let mut envio = EnvioHttps;
    let mut rede = Contador::novo(&mut envio);
    let abertura = leitura(nup, cli.unidade.as_deref(), true);
    let (aberto, acoes) = match abrir_para_escrever(sessao, &abertura, indice, &mut rede) {
        Ok(aberto) => aberto,
        Err(erro) => return saida_erro::erro_fonte_http(
            cli.json, erro.subcodigo(), erro.mensagem(), erro.status_http()
        ),
    };
    let link = match acoes.prazo {
        Ok(Some(link)) => link,
        Ok(None) => return recusar(cli.json, "prazo_nao_publicado_na_barra",
            "a barra do processo não oferece controle de prazo"),
        Err(erro) => return recusar(cli.json, erro.subcodigo(),
            "o link de controle de prazo não corresponde ao processo ou à forma observada"),
    };
    let cookie = match ler_cookie(sessao) {
        Ok(cookie) => cookie,
        Err(erro) => return saida_erro::erro_fonte_http(
            cli.json, erro.subcodigo(), erro.mensagem(), erro.status_http()
        ),
    };
    let (html, _) = match obter_tela(&mut rede, &cookie, &link) {
        Ok(tela) => tela,
        Err(FalhaDaTela::Transporte(erro)) => return saida_erro::erro_fonte_http(
            cli.json, erro.subcodigo(), erro.mensagem(), erro.status_http()
        ),
        Err(FalhaDaTela::Redirecionada) => return recusar(cli.json,
            "tela_de_prazo_redirecionada", "a tela de prazo redirecionou inesperadamente"),
        Err(FalhaDaTela::SessaoEncerrada) => return recusar(cli.json,
            "sessao_encerrada_ao_ler_prazo", "a tela de prazo devolveu o login"),
    };
    let tela = match ler_tela(&html, &aberto.arvore.processo.id) {
        Ok(tela) => tela,
        Err(erro) => return recusar(cli.json, erro.subcodigo(), "tela de prazo fora da forma observada"),
    };
    let atual = match &tela.estado {
        EstadoDoPrazo::SemPrazo => json!({"tipo": "sem_prazo"}),
        EstadoDoPrazo::Data(data) => json!({"tipo": "data", "data": data}),
        EstadoDoPrazo::Concluido => json!({"tipo": "concluido"}),
    };
    let solicitado = match pedido {
        PedidoDePrazo::Ate(data) => json!({"ate": data}),
        PedidoDePrazo::Dias(dias) => json!({"dias": dias}),
        PedidoDePrazo::Ler | PedidoDePrazo::Excluir => Value::Null,
    };
    let campos: Vec<_> = campos_do_pedido(pedido).into_iter().map(|(nome, _)| nome).collect();
    let resultado = json!({
        "nup": nup.as_str(),
        "id_procedimento": aberto.arvore.processo.id,
        "unidade": aberto.unidade,
        "origem": "web",
        "atual": atual,
        "valor_do_checkbox_de_dias_uteis": tela.valor_dos_uteis,
        "solicitado": solicitado,
        "campos_da_previa": campos,
        "confirmado": false,
        "requisicoes": rede.feitas(),
    });
    if cli.json {
        println!("{resultado}");
    } else {
        println!("Processo {} — prazo atual: {}", nup.as_str(), atual);
        if !solicitado.is_null() {
            println!("Prévia: {solicitado}. Para aplicar, --confirmar (ainda bloqueado).");
        }
        println!("Requisições: {}", rede.feitas());
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processo_prazo::ErroPrazo;

    #[test]
    fn tipos_de_entrada_distintos_nao_sao_tratados_como_escrita() {
        assert_eq!(interpretar_pedido(None, None, false), Ok(PedidoDePrazo::Ler));
        assert_eq!(interpretar_pedido(None, None, true), Ok(PedidoDePrazo::Excluir));
        assert_eq!(interpretar_pedido(Some("31/02/2027"), None, false), Err(ErroPrazo::DataInvalida));
    }
}
