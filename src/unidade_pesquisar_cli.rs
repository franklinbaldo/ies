//! #244: leitura de snapshots de unidades, exclusivamente local.
//! A identidade da instalação e da unidade é derivada dos arquivos privados
//! da sessão. URL assinada, cookie e `infra_hash` nunca são publicados.
use crate::configuracao::{self, ArquivosDeAcesso};
use crate::erro_fonte;
use crate::indice_local::{self, IndiceLocal, ListaGuardada};
use crate::instalacao::Instalacao;
use crate::transporte::sessao::ReferenciasDaSessao;
use serde_json::{Value, json};
use std::process::ExitCode;

type ErroLocal = (&'static str, &'static str);

/// A URL privada do Controle contém a unidade ativa (ID opaco). Sem essa
/// informação não se pode misturar snapshots de unidades distintas.
/// Não há pedido HTTP nem login, e a validade remota não é presumida.
fn escopo_da_sessao(arquivos: &ArquivosDeAcesso) -> Result<(String, String), ErroLocal> {
    let referencias =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)
            .map_err(|e| (e.subcodigo(), e.mensagem()))?;
    referencias
        .conferir_controle()
        .map_err(|e| (e.subcodigo(), e.mensagem()))?;
    let instalacao = Instalacao::interpretar(referencias.origem()).ok_or((
        "origem_nao_autorizada",
        "a origem da sessão não identifica uma instalação HTTPS válida",
    ))?;
    let unidade = indice_local::unidade_dos_links(referencias.url_do_controle()).ok_or((
        "unidade_da_sessao_nao_identificada",
        "a URL local do Controle não identifica uma única unidade ativa",
    ))?;
    Ok((instalacao.url_base().clone(), unidade))
}

/// Falha fechada: em uma consulta offline não se confere o rótulo da
/// unidade na tela. A opção global --unidade nunca é ignorada.
fn conferir_unidade_pedida(unidade: Option<&str>) -> Result<(), ErroLocal> {
    if unidade.is_some() {
        Err((
            "unidade_pedida_nao_verificavel_offline",
            "não é possível conferir --unidade sem consultar a tela do SEI",
        ))
    } else {
        Ok(())
    }
}

fn ler_guardadas(
    indice: &IndiceLocal,
    instalacao: &str,
    unidade: &str,
    termo: Option<&str>,
) -> Result<ListaGuardada, ErroLocal> {
    if !indice.ligado() {
        return Err((
            "indice_local_desligado",
            "o índice local está desligado; a leitura Web ainda não foi integrada",
        ));
    }
    let resultado = match termo {
        Some(termo) => indice.pesquisar_unidades_guardadas(instalacao, unidade, termo),
        None => indice.lista_de_unidades(instalacao, unidade),
    };
    if !indice.ligado() {
        return Err((
            "indice_local_indisponivel",
            "não foi possível consultar o índice local; a leitura Web ainda não foi integrada",
        ));
    }
    resultado.ok_or((
        "lista_unidades_nao_guardada",
        "não há snapshot de unidades para esta instalação e unidade; a leitura Web está pendente",
    ))
}

fn json_da_lista(lista: &ListaGuardada, unidade: &str) -> Value {
    let unidades: Vec<Value> = lista
        .unidades
        .iter()
        .map(|u| {
            json!({
                "id": &u.id,
                "sigla": &u.sigla,
                "descricao": &u.descricao,
                "orgao": &u.orgao,
            })
        })
        .collect();
    json!({
        "origem": "indice",
        "unidade_ativa": unidade,
        "lida_em": lista.lida_em,
        "total": lista.total,
        "unidades": unidades,
        "requisicoes": 0,
    })
}

/// Consulta Web completa quando não há snapshot ou --atualizar foi pedido.
/// A falha do índice não impede a leitura; somente impede persistir.
fn atualizar_por_web(
    arquivos: &ArquivosDeAcesso,
    indice: &IndiceLocal,
    instalacao: &str,
    unidade: &str,
) -> Result<(ListaGuardada, usize, bool), ErroLocal> {
    let referencias =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)
            .map_err(|e| (e.subcodigo(), e.mensagem()))?;
    referencias
        .conferir_controle()
        .map_err(|e| (e.subcodigo(), e.mensagem()))?;
    let cookie = referencias
        .cookie()
        .valor()
        .map_err(|e| (e.subcodigo(), e.mensagem()))?;
    let lida = crate::unidade_lupa::ler_integral(
        &mut crate::unidade_lupa::HttpsLupa,
        referencias.url_do_controle(),
        referencias.origem(),
        cookie,
    )
    .map_err(|e| {
        (
            e.subcodigo(),
            "a lupa não devolveu uma lista integral verificável",
        )
    })?;
    let total = lida.total;
    let persistida = indice.guardar_lista_de_unidades(instalacao, unidade, &lida.unidades, total);
    let lida_em = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64());
    let lista = ListaGuardada {
        unidades: lida.unidades,
        total,
        lida_em,
    };
    Ok((lista, lida.requisicoes, persistida))
}

fn executar(
    json_mode: bool,
    unidade_esperada: Option<&str>,
    termo: Option<&str>,
    atualizar: bool,
) -> ExitCode {
    if let Err((subcodigo, mensagem)) = conferir_unidade_pedida(unidade_esperada) {
        return erro_fonte(json_mode, subcodigo, mensagem);
    }
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let (instalacao, unidade) = match escopo_da_sessao(&arquivos) {
        Ok(escopo) => escopo,
        Err((subcodigo, mensagem)) => return erro_fonte(json_mode, subcodigo, mensagem),
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let guardada = if atualizar {
        None
    } else {
        ler_guardadas(&indice, &instalacao, &unidade, termo).ok()
    };
    let (lista, origem, requisicoes, persistida) = if let Some(lista) = guardada {
        (lista, "indice", 0, true)
    } else {
        let (lista, requisicoes, persistida) = match atualizar_por_web(
            &arquivos, &indice, &instalacao, &unidade,
        ) {
            Ok(resultado) => resultado,
            Err((subcodigo, mensagem)) => return erro_fonte(json_mode, subcodigo, mensagem),
        };
        let lista = match termo {
            Some(t) => crate::indice_local::filtrar_lista(lista, t),
            None => lista,
        };
        (lista, "sei", requisicoes, persistida)
    };
    if json_mode {
        let mut valor = json_da_lista(&lista, &unidade);
        valor["origem"] = json!(origem);
        valor["requisicoes"] = json!(requisicoes);
        valor["persistida_no_indice"] = json!(persistida);
        println!("{valor}");
    } else {
        println!(
            "Unidades: {} (origem: {}, leitura: {}, requisições: {})",
            lista.total, origem, lista.lida_em, requisicoes
        );
        if !persistida {
            eprintln!("IES: índice desligado ou indisponível; a lista não foi guardada");
        }
        for item in &lista.unidades {
            println!(
                "{}\t{}\t{}\t{}",
                item.id, item.sigla, item.descricao, item.orgao
            );
        }
    }
    ExitCode::SUCCESS
}

pub(super) fn listar(json_mode: bool, unidade_esperada: Option<&str>, atualizar: bool) -> ExitCode {
    executar(json_mode, unidade_esperada, None, atualizar)
}

pub(super) fn pesquisar(json_mode: bool, unidade_esperada: Option<&str>, termo: &str) -> ExitCode {
    if termo.trim().is_empty() {
        return erro_fonte(
            json_mode,
            "termo_de_unidade_vazio",
            "termo de pesquisa vazio",
        );
    }
    executar(json_mode, unidade_esperada, Some(termo), false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indice_local::UnidadeGuardada;

    #[test]
    fn json_publica_so_snapshot_ja_verificado_e_zero_requisicoes() {
        let lista = ListaGuardada {
            unidades: vec![UnidadeGuardada {
                id: "17".to_owned(),
                sigla: "CDE/SEAD".to_owned(),
                descricao: "Administração".to_owned(),
                orgao: "SEAD".to_owned(),
            }],
            total: 1,
            lida_em: 1234.0,
        };
        let saida = json_da_lista(&lista, "7");
        assert_eq!(saida["origem"], "indice");
        assert_eq!(saida["total"], 1);
        assert_eq!(saida["requisicoes"], 0);
        assert_eq!(saida["unidades"][0]["id"], "17");
        assert!(saida.get("cookie").is_none());
        assert!(saida.get("url").is_none());
    }

    #[test]
    fn filtro_global_de_unidade_nunca_eh_ignorado() {
        assert_eq!(conferir_unidade_pedida(None), Ok(()));
        assert_eq!(
            conferir_unidade_pedida(Some("PPI")).unwrap_err().0,
            "unidade_pedida_nao_verificavel_offline"
        );
    }
}
