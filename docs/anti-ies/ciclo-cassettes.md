---
type: Guide
title: "anti-IES — ciclo de cassettes e esquema SQLite evolutivo"
description: "Contrato de trabalho vertical, banco derivado efêmero e compatibilidade do IES"
---

# Ciclo fechado IES ↔ anti-IES

**Fonte de verdade:** cassettes públicas e imutáveis do SEI, com cards OKF,
ZIPs, hashes e proveniência conferidos. **Resultado esperado:** fatos
extraídos da resposta original por verificação independente do renderizador.
O anti-IES não deve simplesmente devolver o HTML gravado.

1. Carregar a cassette **conferida** (ZIP SHA-1, membro SHA-256 normalizado,
   instalação, schema e status), com o leitor de acervo existente.
2. O **IES real** interpreta a resposta e transforma os fatos observados em
   entidades Rust tipadas; informações que não vieram na cassette continuam
   ausentes, jamais iguais a zero por conveniência.
3. Gravar a projeção num **SQLite efêmero**, criado do zero para a execução
   local/offline. O **anti-IES** lê as entidades e serve a operação Web
   equivalente, sem contatos externos com instalações SEI.
4. O **comando real do IES** consome a resposta do anti-IES. Comparar saída
   e efeitos observáveis com uma expectativa independente da cassette:
   valores, causas de erro, unidade, listas, total, faixa e estado, conforme
   o que a captura demonstrou. HTML byte a byte não é contrato.
5. Só substituir mocks/testes redundantes após comprovar cobertura
   equivalente **inclusive das recusas**. Preservar testes unitários que
   provam invariantes distintas e os gates atuais.

O teste de parser isolado não comprova o passo 4: documentar `PARCIAL`,
não `E2E PASS`. Um caso completo por operação vale mais do que
centenas de fixtures geradas pelo próprio servidor.

## Esquema (shape) declarado e versionado

Novas cassettes podem exigir entidades, tabelas, colunas, chaves,
relacionamentos e índices novos. **Isso é evolução normal da capacidade.**
Não congelar o esquema para evitar alterar IES e anti-IES juntos.

- A **definição canônica** é SQL versionado e revisável em
  `crates/iesei/migrations/NNNN_<capacidade>.sql`, a iniciar quando for
  implementado o construtor desse banco derivado. A crate `iesei`
  registra e aplica as migrações explicitamente, em ordem, usando
  `PRAGMA user_version`. Não dispersar DDL entre parsers, testes e
  handlers HTTP, nem manter catálogos paralelos de colunas.
- Para cada mudança de shape, a **mesma PR vertical** inclui a alteração
  SQL, os tipos Rust consumidores/produtores, a ingestão da cassette,
  a leitura/renderização do anti-IES e a regressão de compatibilidade.
  Campos ausentes são `Option<T>`; alternativas fechadas são `enum`;
  falhas preservam `Result<T,E>` e suas causas, sem strings de status,
  `serde_json::Value` genérico no domínio nem novos subcódigos.
- O banco de **teste** é um artefato descartável: no início de cada ensaio,
  criá-lo a partir do esquema e reconstruí-lo das cassettes. Quando a
  versão mudar, **recriar**, não transportar dados derivados por migração
  histórica por obrigação. Testar pelo menos criação vazia, ingestão
  idempotente, isolamento, dado ausente e reexecução com nova versão.
  Não é necessário cachear o SQLite.
- **Não confundir** esse banco com `ies.sqlite3`, o **índice operacional**
  do IES, que contém inclusive links assinados de sessão. Este último
  tem suas migrações próprias em `src/indice_local/abertura.rs` e
  **não deve ser montado diretamente no anti-IES de testes**. A migração
  operacional `controle_unidade` v4 e `MIGRACAO_CONTROLE` hoje
  existentes não representam o esquema completo do novo banco derivado.
  Reaproveitar tipos de domínio sem compartilhar credenciais nem
  acoplar o experimento ao índice real.

## Cache e proveniência

Os ZIPs originais permanecem na origem pública durável indicada nos cards.
O GitHub Actions pode manter **cache de ZIPs já validados**, com chave
derivada dos hashes registrados; conferir hashes novamente antes de usar.
Cache é acelerador, **não fonte de verdade nem prova de compatibilidade**.
Sua perda apenas exige recuperar os ZIPs e reconstruir o SQLite. É
permitido cachear banco derivado somente se todas as entradas e a versão
do esquema constarem da chave, mas **preferir sempre reconstruí-lo**.
Não comitar cookies, URLs assinadas, `infra_hash`, sessão, HTML privado
nem bancos derivados.

## Trabalho simultâneo e aceite

Um agente pode corrigir o parser do IES, acrescentar `struct`/`enum`,
nova migração SQL, renderização do anti-IES e o teste de CLI **na mesma
PR**, desde que sejam partes necessárias de **uma única fatia observável**
e o ciclo feche. Isso prevalece sobre a divisão por prefixos nessa
vertical; respeitar reservas e branches de outros donos, coordenando
arquivos compartilhados. Não criar uma PR por camada, não introduzir
framework, outra suíte de CI ou testes repetidos.

A PR declara: card/hashes, operação/cenário BDD AIES-01..04 (ou cenário
CLI previamente aprovado), versão do shape e arquivos de migração,
tabelas e tipos novos, fato observado vs. desconhecido, resultados de
`cassette → banco → anti-IES` e `IES → anti-IES`, testes antigos
substituídos e suas garantias, status `PASS/FAIL/NOT_RUN`, SHA e risco
residual. **Sem prova, registrar NOT_RUN, não anunciar compatibilidade.**
CI canônico e limites de treinamento, HTTPS e não produção permanecem.
