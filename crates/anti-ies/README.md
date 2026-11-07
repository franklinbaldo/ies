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
Sem fonte configurada, até `procedimento_controlar` permanece em 501.
A CLI `ies` existente permanece inalterada.

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
- Apenas Controle resumido R observado; não há suporte ao fluxo de abertura
  de processos, documentos, visão D ou escritas. Dados insuficientes são 503.
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
