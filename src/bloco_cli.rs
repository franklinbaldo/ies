//! CLI Web de blocos: GET Controle, GET lista e, para bloco N, GET detalhe.
//! Zero POST, JS, retry ou restauração da tela.
use crate::bloco::{self, ErroBloco, TipoBloco};
use crate::bloco_detalhe::{self, ErroDetalheBloco};
use crate::configuracao;
use crate::controle_reconhecido;
use crate::erro_web::ErroCaixa;
use crate::transporte::{self, FalhaTransporte};
use crate::transporte::envio_post::ler_tela_via;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
use crate::{Bloco, BlocoAcao, Cli, erro_fonte};
use serde_json::{Value, json};
use std::process::ExitCode;

#[derive(Debug)]
enum FalhaLeitura {
    Transporte(FalhaTransporte),
    Controle(ErroCaixa),
    UnidadeInesperada,
    Bloco(ErroBloco),
    Detalhe(ErroDetalheBloco),
}
impl FalhaLeitura {
    fn subcodigo(&self)->&'static str{
        match self {
            Self::Transporte(e)=>e.subcodigo(),
            Self::Controle(e)=>e.subcodigo(),
            Self::UnidadeInesperada=>"unidade_inesperada_na_leitura_de_blocos",
            Self::Bloco(e)=>e.subcodigo(),
            Self::Detalhe(e)=>e.subcodigo(),
        }
    }
    fn mensagem(&self)->&'static str{
        match self {
            Self::Transporte(e)=>e.mensagem(),
            Self::Controle(_)=>"Controle não reconhecido na leitura de blocos",
            Self::UnidadeInesperada=>"unidade atual diferente da solicitada",
            Self::Bloco(e)=>e.mensagem(),
            Self::Detalhe(_)=>"a tabela de documentos ou processos do bloco não foi reconhecida",
        }
    }
    fn status_http(&self)->Option<&transporte::StatusHttpRecusado>{
        match self { Self::Transporte(e)=>e.status_http(), _=>None }
    }
}
/// Transporte injetável, mesmo caminho da produção; cada chamada faz um GET.
fn ler_via<V: EnvioDoControle>(
    sessao:&ReferenciasDaSessao,
    unidade_esperada:Option<&str>,
    tipo:TipoBloco,
    numero:Option<&str>,
    pagina:Option<u32>,
    envio:&mut V,
)->Result<Value,FalhaLeitura>{
    let mut requisicoes=0usize;
    let controle=ler_tela_via(
        sessao.url_do_controle(),sessao.cookie(),sessao.origem(),
        |url,cookie|{ requisicoes+=1; envio.get(url,cookie) },
    ).map_err(FalhaLeitura::Transporte)?;
    let unidade=controle_reconhecido::unidade_do_controle(&controle.html)
        .map_err(FalhaLeitura::Controle)?;
    if unidade_esperada.is_some_and(|v|v.split_whitespace().collect::<Vec<_>>().join(" ")!=unidade){
        return Err(FalhaLeitura::UnidadeInesperada);
    }
    let url_lista=bloco::link_do_menu(
        &controle.instalacao,&controle.url,&controle.html,tipo,
    ).map_err(FalhaLeitura::Bloco)?;
    let cookie=sessao.cookie().valor().map_err(FalhaLeitura::Transporte)?;
    requisicoes+=1;
    let resposta=envio.get(&url_lista,cookie).map_err(FalhaLeitura::Transporte)?;
    let html=transporte::decodificar_resposta(resposta).map_err(FalhaLeitura::Transporte)?;
    let lista=bloco::ler_lista(&html,tipo).map_err(FalhaLeitura::Bloco)?;
    if pagina==Some(1) && lista.lista.faixa_observada.as_ref().is_some_and(|f|f.inicio!=1){
        return Err(FalhaLeitura::Bloco(ErroBloco::PaginaDeBlocosNaoImplementada));
    }
    if let Some(numero)=numero {
        let href=lista.href_do_bloco(numero).map_err(FalhaLeitura::Bloco)?;
        let url=bloco::resolver_link(
            &controle.instalacao,&url_lista,href,"rel_bloco_protocolo_listar",Some(numero),
        ).map_err(FalhaLeitura::Bloco)?;
        requisicoes+=1;
        let resposta=envio.get(&url,cookie).map_err(FalhaLeitura::Transporte)?;
        let html=transporte::decodificar_resposta(resposta).map_err(FalhaLeitura::Transporte)?;
        let detalhe=bloco_detalhe::ler_detalhe(&html,numero,tipo)
            .map_err(FalhaLeitura::Detalhe)?;
        return Ok(json!({"unidade":unidade,"origem":"web","requisicoes":requisicoes,"bloco":detalhe}));
    }
    Ok(json!({"unidade":unidade,"origem":"web","requisicoes":requisicoes,"lista":lista.lista}))
}
fn executar_leitura(cli:&Cli,tipo:TipoBloco,numero:Option<&str>,pagina:Option<u32>)->ExitCode{
    if let Some(numero)=numero {
        if numero.is_empty() || !numero.bytes().all(|b|b.is_ascii_digit()){
            return erro_uso(cli.json,"numero_do_bloco_invalido","número de bloco deve ser decimal");
        }
    }
    if pagina.is_some_and(|p|p!=1){
        return erro_fonte(cli.json,ErroBloco::PaginaDeBlocosNaoImplementada.subcodigo(),
            "paginação de blocos ainda não implementada: nenhuma requisição enviada");
    }
    let arquivos=match configuracao::arquivos_de_acesso(){
        Ok(a)=>a,Err(e)=>return erro_fonte(cli.json,e.subcodigo(),e.mensagem()),
    };
    let sessao=match ReferenciasDaSessao::ler(
        &arquivos.url_controle,&arquivos.origem,&arquivos.cookie
    ){
        Ok(s)=>s,
        Err(e)=>return crate::saida_erro::erro_fonte_http(cli.json,e.subcodigo(),e.mensagem(),e.status_http()),
    };
    match ler_via(&sessao,cli.unidade.as_deref(),tipo,numero,pagina,&mut EnvioHttps){
        Ok(valor)=>{
            if cli.json{
                println!("{}",serde_json::to_string_pretty(&valor).unwrap_or_else(|_|"{}".to_owned()));
            }else if let Some(bloco)=valor.get("bloco"){
                println!("IES: bloco {} ({}) — {} item(ns) observados; {} GET(s).",
                    bloco["numero"],tipo.rotulo(),
                    bloco["itens"].as_array().map_or(0,Vec::len),valor["requisicoes"]);
            }else{
                println!("IES: blocos {}: {} na página observada de {} declarados; {} GET(s).",
                    tipo.rotulo(),valor["lista"]["itens"].as_array().map_or(0,Vec::len),
                    valor["lista"]["total_declarado"],valor["requisicoes"]);
                if let Some(itens)=valor["lista"]["itens"].as_array(){
                    for item in itens { println!("{} | {} | {}",item["numero"],item["estado"],item["descricao"]); }
                }
            }
            ExitCode::SUCCESS
        }
        Err(e)=>crate::saida_erro::erro_fonte_http(cli.json,e.subcodigo(),e.mensagem(),e.status_http()),
    }
}
fn erro_uso(json_mode:bool,subcodigo:&str,mensagem:&str)->ExitCode{
    if json_mode {
        println!("{}",json!({"codigo":"entrada_invalida","subcodigo":subcodigo,
            "erro":mensagem,"retentavel":false}));
    }else{ eprintln!("IES: {mensagem} [{subcodigo}]"); }
    ExitCode::from(2)
}
pub(crate) fn executar(cli:&Cli,bloco:&Bloco)->ExitCode{
    match (&bloco.numero,&bloco.acao){
        (None,Some(BlocoAcao::Listar{tipo,pagina}))=>executar_leitura(cli,*tipo,None,*pagina),
        (Some(numero),None)=>executar_leitura(cli,TipoBloco::Assinatura,Some(numero),None),
        (None,None)=>{ crate::esquema_cli::show_noun_help("bloco"); ExitCode::SUCCESS },
        _=>erro_uso(cli.json,"opcoes_de_bloco_incompativeis","número e listagem não podem ser combinados"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[test]
    fn cli_aceita_listar_tipo_e_pagina_e_numero(){
        assert!(Cli::try_parse_from(["ies","bloco","listar","--tipo","interno","--pagina","1"]).is_ok());
        assert!(Cli::try_parse_from(["ies","bloco","2195"]).is_ok());
        assert!(Cli::try_parse_from(["ies","bloco","listar","--tipo","desconhecido"]).is_err());
        assert!(Cli::try_parse_from(["ies","bloco","listar","--pagina","0"]).is_err());
    }
}
