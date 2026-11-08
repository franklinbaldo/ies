---
type: Guide
title: anti-IES — servidor experimental Rust dentro do IES
description: Escopo inicial, execução local e evolução por contratos observados do SEI
---

# anti-IES

O **anti-IES** é um servidor Rust no mesmo workspace que a CLI `ies`.
O objetivo é reproduzir progressivamente **efeitos e respostas observáveis**
do SEI Web, não clonar seu código PHP. A compatibilidade só existe para
operações explicitamente implementadas e verificadas.

## Executar

```sh
cargo run -p anti-ies -- 8765
curl -i http://127.0.0.1:8765/__anti_ies/health
curl -i 'http://127.0.0.1:8765/sei/controlador.php?acao=procedimento_trabalhar'
cargo test -p anti-ies
```

A primeira consulta retorna HTTP 200 e a identidade do servidor de testes.
A segunda retorna HTTP **501**, pois `procedimento_trabalhar` ainda não é servido.
Sem fonte de cassette configurada, um processo e um tipo são criados
**em memória**, e `procedimento_controlar` sem snapshot continua indisponível (503).
O transporte de produção da CLI `ies` não é alterado.

## Documento interno — simulador sintético com estado compartilhado

Sem variáveis de ambiente, o anti-IES inicializa um SQLite **somente em
memória**, com esquema versionado em
`crates/iesei/migrations/0003_documentos_simulados.sql`. Uma instalação
local de exemplo contém o processo `10`, unidade `UNIDADE TESTE` e
tipo `Despacho` (série `291`). Teste a sequência:

```sh
cargo run -p anti-ies -- 8765
curl -i 'http://127.0.0.1:8765/sei/controlador.php?acao=procedimento_trabalhar&id_procedimento=10'
curl -i 'http://127.0.0.1:8765/sei/controlador.php?acao=documento_escolher_tipo&id_procedimento=10'
curl -i 'http://127.0.0.1:8765/sei/controlador.php?acao=documento_gerar&id_procedimento=10&id_serie=291'
curl -i -X POST --data 'hdnIdSerie=291&hdnFlagDocumentoCadastro=2&rdoNivelAcesso=0&txtDescricao=Ensaio' 'http://127.0.0.1:8765/sei/controlador.php?acao=documento_gerar&id_procedimento=10&id_serie=291'
curl -i 'http://127.0.0.1:8765/sei/controlador.php?acao=arvore_visualizar&id_procedimento=10'
```

As leituras não criam documentos. O POST muda o estado **uma vez**,
responde com 302 para um documento sintético e a releitura mostra o novo
número sem perder os anteriores. Os testes exercitam HTTP, recusa de
método e corpo inválido, isolamento por instalação e cenário, e
reconciliação com o seletor de tipos do IES.

Esta é uma **hipótese sintética testável**; o HTML da lista e da árvore
ainda não é integralmente compatível com o leitor de processos/autenticação
da CLI. O ensaio de ponta a ponta do binário completo e a compatibilidade
com o SEI real ainda não estão comprovados. Não há escrita remota,
execução de REA, autenticação, cookies ou redução da validação TLS real.

## Nova capacidade: Controle R reconstruído de cassette

Uma cassette SHA-verificada é interpretada pelos leitores do IES,
salva no banco efêmero com esquema
\`crates/iesei/migrations/0001_controle_resumido.sql\`.
O anti-IES lê **somente** esse SQLite e serve um GET de Controle R sem
repetir o HTML gravado. A resposta é comparada com a original pelo leitor
usado por \`ies unidade atual\` e pelas tabelas de processos do IES.

Para iniciar o servidor com um banco preparado **de testes**, configurar
\`ANTI_IES_CASSETTE_DB\` (arquivo SQLite derivado),
\`ANTI_IES_CASSETTE_ORIGEM\` (URL-base pública da instalação representada)
e \`ANTI_IES_CASSETTE_CENARIO\` (identificador não secreto da cassette).
O servidor lê com \`SQLITE_OPEN_READ_ONLY\`; **não** apontar para
\`ies.sqlite3\`, que é o índice operacional e pode guardar URLs assinadas.

O teste \`cassette_unb_controle_r_fecha_ciclo_http_e_leitor_da_cli\`
executa a cadeia sobre o ZIP original já usado pelo CI de retornos,
com verificação SHA-1/SHA-256 e sem pedir nada ao SEI. O teste emite
PASS apenas se o GET HTTP do anti-IES reproduzir unidade, listas,
totais e faixas interpretados da resposta gravada.
**Não é ainda um subprocesso da CLI completa com sessão TLS**:
essa prova adicional permanece na #355.

## Limites desta fatia

- Listener **somente** em `127.0.0.1`, sem configuração de interface externa.
- Transporte HTTP mínimo, sequencial e de desenvolvimento, que lê apenas a
  primeira linha da requisição (máximo 4096 bytes). **Não usar em produção**.
- Sem usuários, autenticação, cookies, JS, proxy ou acesso a instalações
  SEI; pode ler um banco **efêmero de testes**, nunca o índice operacional;
  não serve como substituto do servidor oficial.
- Controle resumido e retornos mensais usam observações; o documento interno
  é um simulador SQLite sintético. Não reproduz ainda toda a árvore/visão D
  nem a autenticação da CLI. Dados insuficientes são 503.
- Os testes são sintéticos e offline; validação live: **NOT_RUN**.

## Próxima fatia

1. Escolher uma operação Web somente leitura, observada em cassette do acervo
   e numa instalação de treinamento autorizada.
2. Descrever entradas, estado, erros e estrutura da resposta em contrato OKF;
   manter diferenças por instalação explícitas.
3. Construir modelo de domínio tipado e adaptador HTTP para **aquela**
   operação, com teste RED antes da implementação.
4. Usar o IES como consumidor dos dois lados e comparar resultados
   normalizados; identidade de HTML byte a byte não é pré-requisito.
5. Só quando houver contrato real, avaliar framework HTTP adequado e
   persistência, sem carregar o cliente com dependências de servidor.

Acompanhamento: [issue #350](https://github.com/franklinbaldo/ies/issues/350).

## Segunda vertical: retornos mensais da unidade

A cassette pública `unidade-listar-retornos-unb` contém duas páginas
mensais conferidas. O leitor de produção do IES extrai os retornos, a
projeção SQL v2 armazena o mês e os registros tipados, e o anti-IES serve
`GET controlador.php?acao=retorno_programado_listar&data_inicial=...&data_final=...`
com HTML reconstruído, não copiado. O teste usa duas requisições HTTP reais
e compara o resultado do leitor de retornos com o JSON original do acervo.
Mês não observado é 503, não lista vazia. A execução do comando completo
`ies unidade retornos --mes ...` com sessão segura ainda é NOT_RUN.
