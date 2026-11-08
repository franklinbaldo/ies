---
type: Guide
title: "anti-IES — ciclo de cassettes e esquema SQLite evolutivo"
description: "Contrato de trabalho vertical, banco derivado efêmero e compatibilidade do IES"
---

# Ciclo fechado IES ↔ anti-IES

**Implementar → testar → aprender → corrigir:** o teste também produz
evidência. **Cassette e REA não são pré-requisitos** para iniciar, codificar
ou concluir uma fatia funcional: começar com hipótese testável e fixture
sintética. Cassettes verificadas são referência para confrontar e corrigir
hipóteses depois, não uma condição de autorização para implementar.
O anti-IES não deve simplesmente devolver HTML gravado.
O objetivo é um **sistema funcional, com estado coerente entre operações**.
A reconstrução do comportamento Web e das transições segue o
[guia de engenharia reversa com REA](rea-engenharia-reversa.md):
captura comprovada ≠ hipótese; reproduzir estados e efeitos, não páginas isoladas.

1. Partir do BDD da CLI; escrever um teste RED com a hipótese mais simples.
   Sem cassette, construir fixture sintética e seguir implementando.
2. Implementar parser, tipos Rust e estado, usando testes como evidência de
   funcionamento e fonte de contraexemplos.
3. Recriar SQLite efêmero e servir no anti-IES o comportamento testado,
   sem acesso a instalações reais do SEI.
4. Rodar o IES contra o anti-IES e conferir saída, erros e efeitos com
   expectativa independente do renderizador. Com cassette disponível,
   comparar também a gravação externa e seus hashes.
5. Corrigir e repetir. Só ablar mocks após cobertura equivalente de casos
   positivos e negativos; preservar testes de invariantes distintas.

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

**Responsabilidade transversal de toda a equipe, não um agente novo:**
a cada comando/BDD assumido, o implementador inicia também a vertical
anti-IES. **Não esperar por cassette, REA ou "evidência suficiente".**
Fazer o modelo funcionar com testes e hipóteses que possam mudar.
Não delegar o servidor a uma fila separada: quem altera o parser pode alterar
os tipos `iesei`, a migração SQL, a renderização e o teste diferencial
no **mesmo PR**. 🦅 revisa a correspondência à cassette e a ausência de
resultados fictícios; 🦫 favorece tipos precisos e esquema coeso; 🦡
remove duplicações **após** equivalência e desbloqueia integrações;
🐘 prioriza cenários CLI e coordena reservas. Os demais implementadores
seguem suas áreas — UX, processos, Web e documentos — e não disputam um
mesmo arquivo ativo.

Sem cassette, implementar e testar a melhor versão funcional com fixture
sintética revisável. **Não entregar apenas parecer de falta de evidência.**
Gerar HTML pelo estado do servidor, não copiar um mock estático.
Declarar PASS sintético na prova executada e NOT_RUN apenas na
comparação externa ainda não realizada. Não criar novo gate ou pipeline.

Um agente pode corrigir o parser do IES, acrescentar `struct`/`enum`,
nova migração SQL, renderização do anti-IES e o teste de CLI **na mesma
PR**, desde que sejam partes necessárias de **uma única fatia observável**
e o ciclo feche. Isso prevalece sobre a divisão por prefixos nessa
vertical; respeitar reservas e branches de outros donos, coordenando
arquivos compartilhados. Não criar uma PR por camada, não introduzir
framework, outra suíte de CI ou testes repetidos.

A PR informa BDD, hipótese, testes, schema/tipos, ciclo
`IES → SQLite → anti-IES → IES`, status do CI e riscos. Card e hashes
somente quando disponíveis. **PASS nos testes é evidência suficiente
para continuar implementação e integrar a fatia com gates verdes.**
Não confundir PASS sintético com compatibilidade externa comprovada.
Ausência de cassette/REA **não bloqueia desenvolvimento ou PR**;
HTTPS, restrições de produção e segurança continuam vigentes.
