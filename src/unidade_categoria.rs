//! `ies unidade recebidos` e `ies unidade gerados`: uma lista independente
//! da caixa, a página observada com total e faixa; `--pagina N` acrescenta
//! no máximo um POST (`pagina_web`). Na visão D, troca para R e fica nela.
mod pagina_web;

use crate::indice_local::{self, IndiceLocal};
use crate::paginas::Leitura;
use crate::relogin_cli::{self, NotaDoRelogin};
use crate::{arquivo_caixa, configuracao, paginas};
use serde_json::{Value, json};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CategoriaLista {
    Recebidos,
    Gerados,
}

impl CategoriaLista {
    pub(crate) fn categoria(self) -> crate::dominio::Categoria {
        match self {
            Self::Recebidos => crate::dominio::Categoria::Recebidos,
            Self::Gerados => crate::dominio::Categoria::Gerados,
        }
    }

    pub(crate) fn rotulo(self) -> &'static str {
        self.categoria().rotulo()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaCategoria {
    IdentificacaoDaUnidadeAusenteNoControle,
    /// Total ou faixa da categoria não publicáveis (`conferir_faixa_publicavel`).
    FaixaNaoPublicavel(#[from] crate::erro_web::ErroCaixa),
}

/// A lista da categoria na forma única, com a página observada.
pub(crate) fn selecionar(
    leitura: &Leitura,
    categoria: CategoriaLista,
    origem: &str,
) -> Result<Value, FalhaCategoria> {
    if leitura.unidade.is_none() {
        return Err(FalhaCategoria::IdentificacaoDaUnidadeAusenteNoControle);
    }
    let cobertura = match categoria {
        CategoriaLista::Recebidos => &leitura.recebidos,
        CategoriaLista::Gerados => &leitura.gerados,
    };
    paginas::conferir_faixa_publicavel(categoria.categoria(), cobertura)?;
    Ok(crate::saida::lista_da_categoria(
        leitura,
        categoria.categoria(),
        origem,
    )?)
}

/// Publica ou exporta a lista da categoria a partir do HTML obtido,
/// com a conferência de `--unidade` da leitura da caixa.
#[expect(
    clippy::indexing_slicing,
    reason = "serde_json::Value: leitura ausente dá Null e a escrita é em objeto json! local; projeção de saída a substituir em #733"
)]
pub(crate) fn publicar_lista(
    html: &crate::tela_html::HtmlServido,
    unidade: Option<&str>,
    categoria: CategoriaLista,
    json_mode: bool,
    destino: Option<&Path>,
    (nota, indice): (Option<&NotaDoRelogin>, &IndiceLocal),
) -> anyhow::Result<()> {
    let leitura = paginas::juntar(std::slice::from_ref(html), unidade)?;
    let mut modelo = selecionar(&leitura, categoria, "web")?;
    if let Some(caminho) = destino {
        let bytes = arquivo_caixa::gravar(caminho, &modelo)?;
        if json_mode {
            // O arquivo guarda só a lista; `relogin` vai em stdout.
            let mut valor = json!({
                "arquivo":"gravado",
                "bytes":bytes,
                "escopo":"categoria_verificada",
                "categoria":categoria.rotulo()
            });
            relogin_cli::anotar_com_total(&mut valor, nota);
            indice_local::anexar_avisos(&mut valor, indice);
            println!("{valor}");
        } else {
            relogin_cli::avisar(json_mode, nota);
            indice_local::avisar_no_texto(indice);
            println!("IES: categoria verificada gravada em arquivo privado ({bytes} bytes).");
        }
        return Ok(());
    }
    if json_mode {
        relogin_cli::anotar_com_total(&mut modelo, nota);
        indice_local::anexar_avisos(&mut modelo, indice);
        println!("{}", serde_json::to_string_pretty(&modelo)?);
    } else {
        relogin_cli::avisar(json_mode, nota);
        indice_local::avisar_no_texto(indice);
        let quantidade = modelo["processos"].as_array().map_or(0, Vec::len);
        println!(
            "IES: {}: {quantidade} processo(s) observado(s); total declarado: {}; faixa observada: {}.",
            categoria.rotulo(),
            modelo["total_declarado"],
            match modelo.get("faixa_observada") {
                None => "nenhuma".to_owned(),
                Some(faixa) => format!("{} a {}", faixa["inicio"], faixa["fim"]),
            }
        );
    }
    Ok(())
}

/// Um GET; com `--pagina N`, no máximo um POST do paginador da categoria;
/// sessão na visão D: mais a troca para R, sem volta.
pub(crate) fn listar_categoria(
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&Path>,
    categoria: CategoriaLista,
    pagina: Option<usize>,
) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    let indice = IndiceLocal::da_configuracao(&arquivos);
    // Cada tentativa relê a sessão e colhe na geração gravada agora.
    let (html, nota) = relogin_cli::ler(
        &arquivos,
        &indice,
        unidade,
        pagina_web::ErroPaginaWeb::sessao_encerrada,
        || pagina_web::obter(&arquivos, &indice, unidade, categoria.categoria(), pagina),
    )
    .map_err(relogin_cli::subir)?;
    let rodape = (nota.as_ref(), &indice);
    publicar_lista(&html, unidade, categoria, json_mode, destino, rodape)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginas;

    fn leitura(recebidos_total: usize) -> Leitura {
        let html = crate::tela_html::HtmlServido::de_teste(&format!(
            "<a id='lnkInfraUnidade'>PPI</a>\
             <table id='tblProcessosRecebidos'>\
             <caption>{recebidos_total} registros - 1 a 1):</caption>\
             <tr id='P1'><td><a class='processoVisualizado'>R-0001</a></td></tr></table>\
             <table id='tblProcessosGerados'><caption>1 registros - 1 a 1):</caption>\
             <tr id='P2'><td><a class='processoVisualizado'>G-0002</a></td></tr></table>"
        ));
        paginas::juntar(&[html], None).unwrap()
    }

    #[test]
    fn categoria_nao_publica_processos_sem_unidade_exibida() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
        );
        let leitura = paginas::juntar(&[html], None).unwrap();
        assert_eq!(leitura.unidade, None);
        assert_eq!(
            selecionar(&leitura, CategoriaLista::Gerados, "web"),
            Err(FalhaCategoria::IdentificacaoDaUnidadeAusenteNoControle)
        );
    }

    #[test]
    fn categoria_coberta_preserva_unidade_observada() {
        let modelo = selecionar(&leitura(1), CategoriaLista::Gerados, "web").unwrap();
        assert_eq!(modelo["unidade"], "PPI");
        assert_eq!(modelo["categoria"], "gerados");
    }

    fn aceita(args: &[&str]) -> bool {
        use clap::Parser;
        let mut todos = vec!["ies", "unidade"];
        todos.extend_from_slice(args);
        crate::Cli::try_parse_from(todos).is_ok()
    }

    #[test]
    fn listas_sao_subcomandos_e_listar_processos_pagina_a_visao_detalhada() {
        assert!(aceita(&["recebidos"]));
        assert!(aceita(&["gerados"]));
        assert!(aceita(&["gerados", "--saida", "lista.json"]));
        assert!(aceita(&["colunas"]));
        assert!(!aceita(&["listar-processos", "--pagina-arquivo", "p.html"]));
        assert!(!aceita(&["listar-processos", "--categoria", "gerados"]));
        assert!(aceita(&["listar-processos", "--pagina", "2"]));
        assert!(!aceita(&["gerados", "--pagina-arquivo", "pagina-2.html"]));
        assert!(!aceita(&["gerados", "--categoria", "gerados"]));
    }

    #[test]
    fn gerados_inteiros_sao_consultaveis_mesmo_se_recebidos_paginam() {
        let modelo = selecionar(&leitura(200), CategoriaLista::Gerados, "web").unwrap();
        assert_eq!(modelo["categoria"], "gerados");
        assert_eq!(modelo["total_declarado"], 1);
        assert_eq!(modelo["processos"][0]["numero"], "G-0002");
        assert!(modelo.get("recebidos").is_none());
        assert!(modelo.get("completa").is_none());
    }

    #[test]
    fn categoria_paginada_publica_a_pagina_com_total_e_faixa() {
        let modelo = selecionar(&leitura(200), CategoriaLista::Recebidos, "web").unwrap();
        assert_eq!(modelo["total_declarado"], 200);
        assert_eq!(modelo["faixa_observada"], json!({"inicio": 1, "fim": 1}));
        assert_eq!(modelo["processos"][0]["numero"], "R-0001");
        assert!(modelo.get("completa").is_none());
    }

    #[test]
    fn categoria_com_legenda_sem_faixa_e_total_maior_e_recusada() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<a id='lnkInfraUnidade'>PPI</a>\
             <table id='tblProcessosRecebidos'><caption>5 registros:</caption>\
             <tr id='P1'><td><a class='processoVisualizado'>R-0001</a></td></tr></table>\
             <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
        );
        let leitura = paginas::juntar(&[html], None).unwrap();
        let _erro = selecionar(&leitura, CategoriaLista::Recebidos, "web").unwrap_err();
        assert!(selecionar(&leitura, CategoriaLista::Gerados, "web").is_ok());
    }

    #[test]
    fn pagina_das_listas_exige_inteiro_positivo() {
        for lista in ["recebidos", "gerados"] {
            assert!(aceita(&[lista, "--pagina", "2"]));
            assert!(!aceita(&[lista, "--pagina", "0"]));
            assert!(!aceita(&[lista, "--pagina", "-1"]));
            assert!(!aceita(&[lista, "--pagina", "x"]));
        }
    }

    #[test]
    fn lista_tem_a_forma_unica_da_caixa() {
        let modelo = selecionar(&leitura(200), CategoriaLista::Recebidos, "web").unwrap();
        let chaves: Vec<&str> = modelo
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            chaves,
            [
                "categoria",
                "faixa_observada",
                "origem",
                "processos",
                "total_declarado",
                "unidade"
            ]
        );
        assert_eq!(
            modelo,
            crate::saida::lista_da_categoria(
                &leitura(200),
                crate::dominio::Categoria::Recebidos,
                "web"
            )
            .unwrap()
        );
    }

    fn exportar_lista(
        unidade: Option<&str>,
        categoria: CategoriaLista,
    ) -> (anyhow::Result<()>, Option<Value>) {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<a id='lnkInfraUnidade'>PPI</a>\
             <table id='tblProcessosRecebidos'><caption>(3 registros - 1 a 1):</caption>\
             <tr id='P1'><td><a class='processoVisualizado'>R-0001</a></td></tr></table>\
             <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
        );
        let caminho = std::env::temp_dir().join(format!(
            "ies-lista-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let indice = IndiceLocal::desligado();
        let rodape = (None, &indice);
        let codigo = publicar_lista(&html, unidade, categoria, true, Some(&caminho), rodape);
        let valor = std::fs::read(&caminho)
            .ok()
            .map(|bytes| serde_json::from_slice(&bytes).unwrap());
        let _ = std::fs::remove_file(&caminho);
        (codigo, valor)
    }

    #[test]
    fn lista_confere_unidade_global_antes_de_publicar() {
        let (codigo, valor) = exportar_lista(Some("OUTRA"), CategoriaLista::Recebidos);
        assert_eq!(
            codigo
                .unwrap_err()
                .downcast_ref::<crate::erro_web::ErroCaixa>(),
            Some(&crate::erro_web::ErroCaixa::UnidadeInesperada)
        );
        assert!(valor.is_none());
        let (codigo, valor) = exportar_lista(Some("PPI"), CategoriaLista::Gerados);
        codigo.unwrap();
        let valor = valor.unwrap();
        assert_eq!(valor["categoria"], "gerados");
        assert_eq!(valor["total_declarado"], 0);
        assert!(valor.as_object().unwrap().get("recebidos").is_none());
    }

    #[test]
    fn lista_exportada_e_um_objeto_e_nao_um_array() {
        let (codigo, valor) = exportar_lista(None, CategoriaLista::Recebidos);
        codigo.unwrap();
        let valor = valor.unwrap();
        assert!(valor.is_object());
        assert_eq!(valor["categoria"], "recebidos");
        assert_eq!(valor["faixa_observada"], json!({"inicio": 1, "fim": 1}));
        assert_eq!(valor["processos"][0]["numero"], "R-0001");
    }

    #[test]
    fn categoria_unica_com_todas_posicoes_preserva_numeros() {
        let modelo = selecionar(&leitura(1), CategoriaLista::Recebidos, "web").unwrap();
        assert_eq!(modelo["processos"][0]["numero"], "R-0001");
        assert_eq!(modelo["total_declarado"], 1);
        assert!(modelo.get("gerados").is_none());
    }
}
