//! #244: leitura de snapshots de unidades, exclusivamente local.
//! A identidade da instalação e da unidade é derivada dos arquivos privados
//! da sessão. URL assinada, cookie e `infra_hash` nunca são publicados.
use crate::configuracao::{self, ArquivosDeAcesso};
use crate::indice_local::{self, IndiceLocal, ListaGuardada};
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::unidade_lupa::FalhaDaLupa;
use serde_json::{Value, json};

/// Por que a lista de unidades não foi publicada.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDasUnidades {
    Sessao(#[from] FalhaTransporte),
    /// A origem da sessão não identifica uma instalação HTTPS válida.
    OrigemNaoAutorizada,
    /// A URL local do Controle não identifica uma única unidade ativa.
    UnidadeDaSessaoNaoIdentificada,
    /// `--unidade` não se confere sem consultar a tela do SEI.
    UnidadePedidaNaoVerificavelOffline,
    IndiceLocalDesligado,
    IndiceLocalIndisponivel,
    ListaDeUnidadesNaoGuardada,
    /// A lupa não devolveu lista integral; as tentativas feitas vão junto.
    LupaSemListaIntegral {
        #[source]
        falha: FalhaDaLupa,
        requisicoes: usize,
    },
}

/// A URL privada do Controle contém a unidade ativa (ID opaco). Sem essa
/// informação não se pode misturar snapshots de unidades distintas.
/// Não há pedido HTTP nem login, e a validade remota não é presumida.
fn escopo_da_sessao(arquivos: &ArquivosDeAcesso) -> Result<(String, String), ErroDasUnidades> {
    let referencias =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    referencias.conferir_controle()?;
    let instalacao = Instalacao::interpretar(referencias.origem())
        .ok_or(ErroDasUnidades::OrigemNaoAutorizada)?;
    let unidade = indice_local::unidade_dos_links(referencias.url_do_controle())
        .ok_or(ErroDasUnidades::UnidadeDaSessaoNaoIdentificada)?;
    Ok((instalacao.url_base().clone(), unidade))
}

/// Falha fechada: em uma consulta offline não se confere o rótulo da
/// unidade na tela. A opção global --unidade nunca é ignorada.
fn conferir_unidade_pedida(unidade: Option<&str>) -> Result<(), ErroDasUnidades> {
    if unidade.is_some() {
        Err(ErroDasUnidades::UnidadePedidaNaoVerificavelOffline)
    } else {
        Ok(())
    }
}

fn ler_guardadas(
    indice: &IndiceLocal,
    instalacao: &str,
    unidade: &str,
    termo: Option<&str>,
) -> Result<ListaGuardada, ErroDasUnidades> {
    if !indice.ligado() {
        return Err(ErroDasUnidades::IndiceLocalDesligado);
    }
    let resultado = match termo {
        Some(termo) => indice.pesquisar_unidades_guardadas(instalacao, unidade, termo),
        None => indice.lista_de_unidades(instalacao, unidade),
    };
    if !indice.ligado() {
        return Err(ErroDasUnidades::IndiceLocalIndisponivel);
    }
    resultado.ok_or(ErroDasUnidades::ListaDeUnidadesNaoGuardada)
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
/// As falhas de preflight contam zero; as da rede preservam tentativas feitas.
fn atualizar_por_web(
    arquivos: &ArquivosDeAcesso,
    indice: &IndiceLocal,
    instalacao: &str,
    unidade: &str,
) -> Result<(ListaGuardada, usize, bool), ErroDasUnidades> {
    let lupa = |falha, requisicoes| ErroDasUnidades::LupaSemListaIntegral { falha, requisicoes };
    let referencias =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)
            .map_err(|e| lupa(FalhaDaLupa::Transporte(e), 0))?;
    referencias
        .conferir_controle()
        .map_err(|e| lupa(FalhaDaLupa::Transporte(e), 0))?;
    let cookie = referencias
        .cookie()
        .valor()
        .map_err(|e| lupa(FalhaDaLupa::Transporte(e), 0))?;
    let mut requisicoes = 0;
    let lida = crate::unidade_lupa::ler_integral_com_contador(
        &mut crate::unidade_lupa::HttpsLupa,
        referencias.url_do_controle(),
        referencias.origem(),
        cookie,
        &mut requisicoes,
    )
    .map_err(|e| lupa(e, requisicoes))?;
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

#[expect(
    clippy::indexing_slicing,
    reason = "serde_json::Value: leitura ausente dá Null e a escrita é em objeto json! local; projeção de saída a substituir em #733"
)]
fn executar(
    json_mode: bool,
    unidade_esperada: Option<&str>,
    termo: Option<&str>,
    atualizar: bool,
) -> anyhow::Result<()> {
    conferir_unidade_pedida(unidade_esperada)?;
    let arquivos = configuracao::arquivos_de_acesso()?;
    let (instalacao, unidade) = escopo_da_sessao(&arquivos)?;
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let guardada = if atualizar {
        None
    } else {
        ler_guardadas(&indice, &instalacao, &unidade, termo).ok()
    };
    let (lista, origem, requisicoes, persistida) = if let Some(lista) = guardada {
        (lista, "indice", 0, true)
    } else {
        let (lista, requisicoes, persistida) =
            atualizar_por_web(&arquivos, &indice, &instalacao, &unidade)?;
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
    Ok(())
}

pub(super) fn listar(
    json_mode: bool,
    unidade_esperada: Option<&str>,
    atualizar: bool,
) -> anyhow::Result<()> {
    executar(json_mode, unidade_esperada, None, atualizar)
}

/// `termo` não vazio: conferido pelo clap.
pub(super) fn pesquisar(
    json_mode: bool,
    unidade_esperada: Option<&str>,
    termo: &str,
) -> anyhow::Result<()> {
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
            conferir_unidade_pedida(Some("PPI")),
            Err(ErroDasUnidades::UnidadePedidaNaoVerificavelOffline)
        );
    }
}
