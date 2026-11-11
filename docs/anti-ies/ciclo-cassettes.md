---
type: Guide
title: "anti-IES — ciclo de cassettes e esquema SQLite evolutivo"
description: "Contrato de trabalho vertical, banco derivado efêmero e compatibilidade do IES"
---

# Ciclo fechado IES ↔ anti-IES

**Cassette-first no desenvolvimento, replay obrigatório no aceite da PR.** Comece pela operação contratada da CLI e procure as
cassettes já registradas; observações verificadas são a entrada preferida
para modelar o estado inicial do anti-IES. **Não exija** captura inexistente
nem live para começar: quando houver lacunas, use hipótese mínima.
Para integrar uma PR, execute os replays originais disponíveis no GitHub
Actions sobre o head final, independentemente do ensaio sintético. Testes sintéticos auxiliam a modelar transições não capturadas, mas não substituem E2E alimentado por cassettes disponíveis. Não criar novos BDDs, steps ou arquivos Gherkin.
O anti-IES não deve simplesmente devolver HTML gravado.
O objetivo é um **sistema funcional, com estado coerente entre operações**.
A reconstrução do comportamento Web e das transições segue o
[guia de engenharia reversa com REA](rea-engenharia-reversa.md):
captura comprovada ≠ hipótese; reproduzir estados e efeitos, não páginas isoladas.

1. **Operação CLI → busca de observações:** procure o comando em
   [docs/cassettes.md](../cassettes.md), `cassettes/` e
   `contracts/evidencias/`; leia os cards pertinentes ANTES da fixture.
2. **Obter/validar:** confira ZIP e membro JSON por hash. Se o sandbox
   estiver sem rede, execute o
   [workflow de inspeção](../../.github/workflows/inspecionar-cassettes.yml)
   no GitHub Actions (despacho ou arquivo temporário na PR); **não**
   trate restrição do sandbox como ausência de cassette.
3. **Estado inicial:** derive os dados do SQLite efêmero das observações
   disponíveis. Preencha somente campos/efeitos não registrados com
   hipóteses e fixture sintética mínima. Sem cassette aplicável, prossiga
   com hipótese testável e RED; não pare esperando observação externa.
4. **Vertical:** implemente leitores IES, tipos Rust, SQLite e rotas
   anti-IES; execute a CLI real contra o simulador e confira efeitos.
5. **Confronto independente:** rode o leitor IES sobre respostas originais
   e compare separadamente o anti-IES com a mesma observação. Não
   equipare concordância IES↔anti-IES à fidelidade aos cassettes.
   Corrija divergências e preserve invariantes/negativos úteis.

O teste de parser isolado não comprova o passo 4: documentar `PARCIAL`,
não `E2E PASS`. Um caso completo por operação vale mais do que
centenas de fixtures geradas pelo próprio servidor.

## Desenvolvimento duplo e três provas independentes

**Regra para cada operação de comportamento SEI da CLI:** IES e anti-IES
evoluem na **mesma fatia/PR**. O IES executa o comando real; o anti-IES
oferece o comportamento correspondente com estado SQLite efêmero e tipos
`iesei`. A mesma operação deve alterar/consultar a mesma identidade e
produzir efeitos coerentes em comandos posteriores. Reaproveitar partes
já adequadas, sem copiar implementação, duplicar modelos ou mudar arquivos
sem necessidade. Correções puramente locais podem reutilizar o simulador
existente, mas devem exercitar a compatibilidade que afetarem.

Cada capacidade mantém **três resultados distintos**, sempre associados
ao comando e contrato da CLI, ao SHA e à evidência executada:

1. **IES ↔ anti-IES — E2E funcional.** Executar a **CLI
   real** contra o servidor anti-IES (HTTP/TLS em loopback), com o mesmo
   SQLite compartilhado durante a sequência. Semear primeiro a partir
   das observações originais que existirem; nunca substituir uma
   cassette acessível por páginas inventadas. Verificar saída, arquivos,
   estados antes/depois, recusas e causalidade por expectativa independente
   do renderizador: prévia sem mutação, uma confirmação quando aplicável,
   releitura independente e nenhum retry. Testar também uma operação
   seguinte que dependa do estado alterado. Parser isolado não é PASS aqui.
2. **IES e anti-IES ↔ cassettes — duas confrontações independentes.**
   Verificar integridade e proveniência da cassette; rodar **o leitor
   real do IES** nas respostas originais e confrontar **o anti-IES**
   com o mesmo cenário observado. A saída simulada deve corresponder aos
   estados, operações oferecidas, status, campos, recusas e efeitos
   registrados, não apenas satisfazer o próprio parser do IES.
   Comparar sem exigir igualdade byte a byte de tokens, hashes de sessão,
   identificadores transitórios ou HTML irrelevante. Para escritas,
   confrontar também a resposta e a transição de estado/releitura.
   Uma cassette de uma instalação não valida todas as versões do SEI.
3. **IES e anti-IES ↔ SEI de treinamento — validação externa.**
   Executar a mesma operação **somente em instalação autorizada**, com
   versão/unidade identificadas; comparar o resultado real com o
   contrato e com a hipótese do anti-IES. Leitura/GET pode ter efeito.
   Escrita exige prévia → `--confirmar` uma vez → releitura
   independente, com orçamento de requisições declarado e resíduo
   registrado; nunca produção, credenciais em log nem retry da escrita. Divergência real gera correção
   da fatia IES/anti-IES e regressão de cassette quando aplicável.

**A concordância entre os dois programas não prova os níveis 2 ou 3.**
Também não basta IES ler uma cassette: a simulação deve ser confrontada
**separadamente** com a fonte. Não atribuir um PASS antigo à implementação
nova sem cobrir seu comportamento e SHA/versão correspondentes.

**Cadência:** procurar primeiro os cards disponíveis. No GitHub Actions,
executar em cada PR os replays do acervo original e as comparações
independentes aplicáveis no head que será integrado, não só inspecionar
hashes. O workflow de inspeção é auxiliar, não prova semântica.
Se a transição não estiver capturada, avance com fixture mínima e
registre essa comparação `NOT_RUN`, mas exija PASS dos replays existentes
antes do merge. Não exigir CI em cada commit WIP, somente no head final.
O nível 3 acontece quando houver ambiente autorizado e acessível;
`NOT_RUN` não é fracasso sintético, nem PASS real. O objetivo é
compatibilidade observável, sem igualdade fixa de GETs/POSTs como
critério de paridade; orçamento de ensaio e POST mutador único permanecem.

**Registro mínimo na PR/issue**, sem relatório cerimonial: `comando/contrato` +
`IES↔anti-IES: PASS/FAIL/NOT_RUN (teste/CI)` +
`ambos↔cassette: PASS/FAIL/NOT_RUN (card/hash/replay)` +
`treinamento: PASS/FAIL/NOT_RUN (instalação/relatório)`.
`PASS` exige execução real daquela comparação, `FAIL` exige
contraexemplo verificável; `NOT_RUN` diz por quê. **Integrar fatia
sintética verde** respeitando os gates oficiais; não declarar
compatibilidade externa concluída antes dos níveis 2 e 3.
A próxima rodada corrige o primeiro contraexemplo, não reinicia a
capacidade nem aguarda uma prova inexistente.

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
a cada operação assumida, o implementador inicia também a vertical
anti-IES. **Não esperar por cassette, REA ou "evidência suficiente".**
Fazer o modelo funcionar com testes e hipóteses que possam mudar.
Não delegar o servidor a uma fila separada: quem altera o parser pode alterar
os tipos `iesei`, a migração SQL, a renderização e o teste diferencial
no **mesmo PR**. 🦅 revisa a correspondência à cassette e a ausência de
resultados fictícios; 🦫 favorece tipos precisos e esquema coeso; 🦡
remove duplicações **após** equivalência e desbloqueia integrações;
🐘 prioriza operações contratadas e coordena a fila. Os demais implementadores
seguem suas áreas — UX, processos, Web e documentos — e não disputam um
mesmo arquivo ativo.

Sem cassette, implementar e testar a melhor versão funcional com fixture
sintética revisável. **Não entregar apenas parecer de falta de evidência.**
Gerar HTML pelo estado do servidor, não copiar um mock estático.
Declarar PASS sintético na prova executada e NOT_RUN apenas na
comparação externa ainda não realizada. Não criar novo gate ou pipeline.

Um agente pode corrigir o parser do IES, acrescentar `struct`/`enum`,
nova migração SQL, renderização do anti-IES e o teste de CLI **na mesma
PR**, desde que sejam partes necessárias de **uma única fatia E2E observável**
e o ciclo feche. Isso prevalece sobre a divisão por prefixos nessa
vertical; respeitar reservas e branches de outros donos, coordenando
arquivos compartilhados. Não criar uma PR por camada, não introduzir
framework, outra suíte de CI ou testes repetidos.

A PR informa comando/contrato, cassette, hipótese residual, testes, schema/tipos, ciclo
`IES → SQLite → anti-IES → IES`, status do CI e riscos. Card e hashes
somente quando disponíveis. **PASS nos testes é evidência suficiente para continuar implementação.
A integração exige também replay de cassettes disponíveis aprovado no
GitHub Actions para o head final.**
Não confundir PASS sintético com compatibilidade externa comprovada.
Ausência de cassette/REA **não bloqueia desenvolvimento ou PR**;
HTTPS, restrições de produção e segurança continuam vigentes.
